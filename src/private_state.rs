//! Read-only startup checks for an administratively provisioned private tree.
//! This does not repair permissions, inspect account membership, or establish a
//! service sandbox. Ancestor directories and permitted principals must be trusted.

use std::fs::File;
use std::path::Path;

const MAX_ENTRIES: usize = 1_024;
const MAX_DEPTH: usize = 32;

#[derive(Debug)]
pub struct PrivateStatePolicy {
    inner: platform::Policy,
}

impl PrivateStatePolicy {
    /// Extra Windows SIDs come only from trusted operator configuration, never
    /// service requests. They permit retained operator access alongside the
    /// current process, SYSTEM, and Administrators. Unix accepts no extra SIDs.
    pub fn current(additional_trusted_sids: &[String]) -> Result<Self, String> {
        Ok(Self {
            inner: platform::Policy::current(additional_trusted_sids)?,
        })
    }
}

#[derive(Debug)]
pub struct PrivateStateReport {
    pub checked_entries: usize,
    pub owner: String,
    /// Retain this report through controller shutdown. Windows deliberately
    /// omits delete sharing from the inspected root handle.
    _root: File,
}

struct Inspected {
    file: File,
    directory: bool,
    owner: String,
}

/// Check an existing tree before opening journals, verifiers, backups or exports.
/// No file contents are read. Every existing descendant is checked; protecting
/// only the directory would miss independently permissive child descriptors.
/// The bound covers traversal/memory, not worst-case filesystem syscall latency.
pub fn validate_private_tree(
    path: &Path,
    policy: &PrivateStatePolicy,
) -> Result<PrivateStateReport, String> {
    if !path.is_absolute() {
        return Err("private_state_requires_absolute_path".into());
    }
    let root = platform::inspect(path, &policy.inner, true)?;
    if !root.directory {
        return Err("private_state_requires_directory".into());
    }
    let mut pending = vec![(path.to_path_buf(), 0_usize)];
    let mut count = 1;
    while let Some((directory, depth)) = pending.pop() {
        let entries = std::fs::read_dir(directory).map_err(|_| "private_state_listing_failed")?;
        for entry in entries {
            if count >= MAX_ENTRIES {
                return Err("private_state_entry_limit".into());
            }
            let entry = entry.map_err(|_| "private_state_listing_failed")?;
            let inspected = platform::inspect(&entry.path(), &policy.inner, false)?;
            count += 1;
            if inspected.directory {
                if depth + 1 >= MAX_DEPTH {
                    return Err("private_state_depth_limit".into());
                }
                pending.push((entry.path(), depth + 1));
            }
        }
    }
    Ok(PrivateStateReport {
        checked_entries: count,
        owner: root.owner,
        _root: root.file,
    })
}

#[cfg(windows)]
mod platform {
    use super::*;
    use std::collections::BTreeSet;
    use std::ffi::c_void;
    use std::os::windows::fs::OpenOptionsExt;
    use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
    use std::ptr::{addr_of_mut, null_mut};
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Authorization::{
        ConvertSidToStringSidW, ConvertStringSidToSidW, GetSecurityInfo, SE_FILE_OBJECT,
    };
    use windows_sys::Win32::Security::{
        ACCESS_ALLOWED_ACE, ACE_HEADER, DACL_SECURITY_INFORMATION, GetAce, GetLengthSid,
        GetSecurityDescriptorControl, GetSecurityDescriptorDacl, GetSecurityDescriptorLength,
        GetSecurityDescriptorOwner, GetTokenInformation, IsValidAcl, IsValidSecurityDescriptor,
        IsValidSid, OWNER_SECURITY_INFORMATION, SE_DACL_PROTECTED, TOKEN_QUERY, TOKEN_USER,
        TokenUser,
    };
    use windows_sys::Win32::Storage::FileSystem::{
        BY_HANDLE_FILE_INFORMATION, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT,
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_READ_ATTRIBUTES,
        FILE_SHARE_READ, FILE_SHARE_WRITE, GetFileInformationByHandle, READ_CONTROL,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    const MAX_ACES: u16 = 64;
    const MAX_SID_TEXT: usize = 184;

    #[derive(Debug)]
    pub struct Policy {
        permitted: BTreeSet<String>,
    }

    // Own only allocations returned by documented LocalAlloc-based Win32 APIs.
    struct LocalAllocation(*mut c_void);
    impl Drop for LocalAllocation {
        fn drop(&mut self) {
            if !self.0.is_null() {
                // SAFETY: the allocation is uniquely owned and released once.
                unsafe { LocalFree(self.0) };
            }
        }
    }

    impl Policy {
        pub fn current(extra: &[String]) -> Result<Self, String> {
            if extra.len() > 8 {
                return Err("private_state_principal_limit".into());
            }
            let mut permitted = BTreeSet::from([
                current_sid()?,
                "S-1-5-18".into(),     // Local System.
                "S-1-5-32-544".into(), // Builtin Administrators.
            ]);
            for text in extra {
                if text.len() > MAX_SID_TEXT || !text.starts_with("S-") || text.contains('\0') {
                    return Err("private_state_invalid_sid".into());
                }
                let wide: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
                let mut sid = null_mut();
                // SAFETY: the input is terminated; the API initializes sid on success.
                if unsafe { ConvertStringSidToSidW(wide.as_ptr(), &mut sid) } == 0 {
                    return Err("private_state_invalid_sid".into());
                }
                let owned = LocalAllocation(sid);
                permitted.insert(sid_string(owned.0)?);
            }
            Ok(Self { permitted })
        }
    }

    fn current_sid() -> Result<String, String> {
        let mut token = null_mut();
        // SAFETY: process pseudo-handle is valid; output points to a local variable.
        if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
            return Err("private_state_token_unavailable".into());
        }
        // SAFETY: OpenProcessToken returned a new owned token handle.
        let token = unsafe { OwnedHandle::from_raw_handle(token) };
        let mut needed = 0;
        // SAFETY: zero-capacity query only obtains the required buffer length.
        unsafe {
            GetTokenInformation(token.as_raw_handle(), TokenUser, null_mut(), 0, &mut needed)
        };
        if !(std::mem::size_of::<TOKEN_USER>() as u32..=4096).contains(&needed) {
            return Err("private_state_token_invalid".into());
        }
        // usize storage provides TOKEN_USER's pointer alignment.
        let mut buffer = vec![0_usize; (needed as usize).div_ceil(std::mem::size_of::<usize>())];
        // SAFETY: aligned allocation contains at least needed bytes; token stays open.
        if unsafe {
            GetTokenInformation(
                token.as_raw_handle(),
                TokenUser,
                buffer.as_mut_ptr().cast(),
                needed,
                &mut needed,
            )
        } == 0
        {
            return Err("private_state_token_unavailable".into());
        }
        // SAFETY: successful TokenUser query initialized this aligned TOKEN_USER.
        let user = unsafe { &*buffer.as_ptr().cast::<TOKEN_USER>() };
        sid_string(user.User.Sid)
    }

    fn sid_string(sid: *mut c_void) -> Result<String, String> {
        // SAFETY: callers supply SID pointers owned by live OS-produced descriptors.
        if sid.is_null() || unsafe { IsValidSid(sid) } == 0 {
            return Err("private_state_invalid_descriptor".into());
        }
        let mut text = null_mut();
        // SAFETY: the SID was validated and the output pointer is writable.
        if unsafe { ConvertSidToStringSidW(sid, &mut text) } == 0 {
            return Err("private_state_invalid_descriptor".into());
        }
        let _owned = LocalAllocation(text.cast());
        let mut length = 0;
        // Windows SID strings have a fixed structural maximum; the API supplies
        // a terminated allocation. Do not scan arbitrary caller text.
        while length <= MAX_SID_TEXT && unsafe { *text.add(length) } != 0 {
            length += 1;
        }
        if length > MAX_SID_TEXT {
            return Err("private_state_invalid_descriptor".into());
        }
        // SAFETY: length is within the allocated, terminated API result.
        String::from_utf16(unsafe { std::slice::from_raw_parts(text, length) })
            .map_err(|_| "private_state_invalid_descriptor".into())
    }

    fn descriptor_owner(
        descriptor: &LocalAllocation,
        policy: &Policy,
        require_protected: bool,
        directory: bool,
    ) -> Result<String, String> {
        let sd = descriptor.0;
        // SAFETY: this pointer is an owned descriptor returned by the Windows API.
        if sd.is_null()
            || unsafe { IsValidSecurityDescriptor(sd) } == 0
            || unsafe { GetSecurityDescriptorLength(sd) } > 65_536
        {
            return Err("private_state_invalid_descriptor".into());
        }
        let (mut control, mut revision, mut owner, mut defaulted) = (0, 0, null_mut(), 0);
        // SAFETY: output pointers refer to valid local variables; sd remains alive.
        if unsafe { GetSecurityDescriptorControl(sd, &mut control, &mut revision) } == 0
            || unsafe { GetSecurityDescriptorOwner(sd, &mut owner, &mut defaulted) } == 0
        {
            return Err("private_state_invalid_descriptor".into());
        }
        if require_protected && control & SE_DACL_PROTECTED == 0 {
            return Err("private_state_inherits_parent_permissions".into());
        }
        let owner = sid_string(owner)?;
        if !policy.permitted.contains(&owner) {
            return Err("private_state_untrusted_owner".into());
        }
        let (mut present, mut dacl) = (0, null_mut());
        // SAFETY: valid descriptor and local out parameters.
        if unsafe { GetSecurityDescriptorDacl(sd, &mut present, &mut dacl, &mut defaulted) } == 0
            || present == 0
            || dacl.is_null()
            || unsafe { IsValidAcl(dacl) } == 0
        {
            return Err("private_state_missing_dacl".into());
        }
        // SAFETY: the OS validated the ACL and its header lies inside sd.
        let count = unsafe { (*dacl).AceCount };
        if count > MAX_ACES {
            return Err("private_state_ace_limit".into());
        }
        let mut has_inheritable_allow = false;
        for index in 0..u32::from(count) {
            let mut ace = null_mut();
            // SAFETY: index is bounded by the validated ACL header count.
            if unsafe { GetAce(dacl, index, &mut ace) } == 0 || ace.is_null() {
                return Err("private_state_invalid_descriptor".into());
            }
            // SAFETY: GetAce returned an ACE in the live validated ACL.
            let header = unsafe { &*ace.cast::<ACE_HEADER>() };
            // SDK ACE types 0/1 are the simple access-allowed/access-denied forms.
            // Object/callback/conditional ACEs need a separately reviewed policy.
            if !matches!(header.AceType, 0 | 1) {
                return Err("private_state_unsupported_ace".into());
            }
            if usize::from(header.AceSize) < std::mem::size_of::<ACCESS_ALLOWED_ACE>() {
                return Err("private_state_invalid_descriptor".into());
            }
            let simple = ace.cast::<ACCESS_ALLOWED_ACE>();
            // SAFETY: both supported ACE forms have this fixed prefix and SID tail.
            let sid = unsafe { addr_of_mut!((*simple).SidStart).cast::<c_void>() };
            if unsafe { IsValidSid(sid) } == 0
                || unsafe { GetLengthSid(sid) } as usize + 8 > usize::from(header.AceSize)
            {
                return Err("private_state_invalid_descriptor".into());
            }
            if header.AceType == 0 && !policy.permitted.contains(&sid_string(sid)?) {
                return Err("private_state_untrusted_allow_ace".into());
            }
            // Object/container inheritance must cover new sidecars and child
            // directories; otherwise Windows can fall back to a creator's
            // default descriptor despite this existing object's private DACL.
            has_inheritable_allow |= header.AceType == 0 && header.AceFlags & 0x07 == 0x03;
        }
        if directory && !has_inheritable_allow {
            return Err("private_state_missing_inheritance".into());
        }
        Ok(owner)
    }

    pub(super) fn inspect(path: &Path, policy: &Policy, root: bool) -> Result<Inspected, String> {
        // OPEN_REPARSE_POINT inspects the named entry itself, never its target.
        // A curated, stable parent tree is still required; this is not NtCreateFile
        // relative traversal or protection from arbitrary concurrent OS mutation.
        let file = File::options()
            .access_mode(READ_CONTROL | FILE_READ_ATTRIBUTES)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)
            .map_err(|_| "private_state_open_failed")?;
        let mut information = BY_HANDLE_FILE_INFORMATION::default();
        // SAFETY: open handle and correctly sized output structure.
        if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut information) } == 0 {
            return Err("private_state_metadata_failed".into());
        }
        if information.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err("private_state_reparse_point".into());
        }
        let directory = information.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0;
        if !directory && information.nNumberOfLinks != 1 {
            return Err("private_state_hard_link".into());
        }
        let mut descriptor = null_mut();
        // SAFETY: handle grants READ_CONTROL; only descriptor output is requested.
        let status = unsafe {
            GetSecurityInfo(
                file.as_raw_handle(),
                SE_FILE_OBJECT,
                OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
                null_mut(),
                null_mut(),
                null_mut(),
                null_mut(),
                &mut descriptor,
            )
        };
        if status != 0 {
            return Err("private_state_descriptor_unavailable".into());
        }
        let descriptor = LocalAllocation(descriptor);
        let owner = descriptor_owner(&descriptor, policy, root, directory)?;
        Ok(Inspected {
            file,
            directory,
            owner,
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
        use windows_sys::Win32::Security::Authorization::ConvertStringSecurityDescriptorToSecurityDescriptorW;
        use windows_sys::Win32::Security::SECURITY_ATTRIBUTES;
        use windows_sys::Win32::Storage::FileSystem::{
            CREATE_NEW, CreateDirectoryW, CreateFileW, FILE_ATTRIBUTE_NORMAL,
        };

        fn descriptor(sddl: &str) -> LocalAllocation {
            let text: Vec<u16> = sddl.encode_utf16().chain(Some(0)).collect();
            let mut descriptor = null_mut();
            // SAFETY: terminated test fixture string and valid output storage.
            assert_ne!(
                unsafe {
                    ConvertStringSecurityDescriptorToSecurityDescriptorW(
                        text.as_ptr(),
                        1,
                        &mut descriptor,
                        null_mut(),
                    )
                },
                0
            );
            LocalAllocation(descriptor)
        }

        fn check(sddl: &str) -> Result<String, String> {
            descriptor_owner(
                &descriptor(sddl),
                &Policy::current(&[]).unwrap(),
                true,
                true,
            )
        }

        #[test]
        fn simple_private_descriptors_pass_and_broad_or_unprotected_ones_fail() {
            assert!(check("O:SYD:P(A;OICI;FA;;;SY)(A;OICI;FA;;;BA)").is_ok());
            assert_eq!(
                check("O:SYD:P").unwrap_err(),
                "private_state_missing_inheritance",
                "an empty DACL denies access but cannot establish sidecar inheritance"
            );
            for (sddl, reason) in [
                (
                    "O:SYD:(A;OICI;FA;;;SY)",
                    "private_state_inherits_parent_permissions",
                ),
                (
                    "O:SYD:P(A;OICI;FR;;;WD)",
                    "private_state_untrusted_allow_ace",
                ),
                (
                    "O:SYD:P(A;OICI;FA;;;BU)",
                    "private_state_untrusted_allow_ace",
                ),
                ("O:WDD:P(A;OICI;FA;;;SY)", "private_state_untrusted_owner"),
                ("O:SYD:PNO_ACCESS_CONTROL", "private_state_missing_dacl"),
                (
                    "O:SYD:P(OA;OICI;FA;00112233-4455-6677-8899-aabbccddeeff;;SY)",
                    "private_state_unsupported_ace",
                ),
                (
                    "O:SYD:P(A;OICINP;FA;;;SY)",
                    "private_state_missing_inheritance",
                ),
            ] {
                assert_eq!(check(sddl), Err(reason.into()), "{sddl}");
            }
        }

        #[test]
        fn trusted_extra_sid_is_explicit_and_inputs_are_bounded() {
            let text = "S-1-5-21-1-2-3-1001".to_owned();
            assert!(
                Policy::current(std::slice::from_ref(&text))
                    .unwrap()
                    .permitted
                    .contains(&text)
            );
            assert!(Policy::current(&["Everyone".into()]).is_err());
            assert!(Policy::current(&vec![text; 9]).is_err());
        }

        #[test]
        fn native_private_tree_passes_and_broad_children_and_hard_links_fail() {
            // Only fresh synthetic objects are provisioned. Existing directories
            // and their ACLs are never changed by this test.
            let base =
                std::env::temp_dir().join(format!("kinesin-private-{}", uuid::Uuid::new_v4()));
            std::fs::create_dir(&base).unwrap();
            struct Cleanup(std::path::PathBuf);
            impl Drop for Cleanup {
                fn drop(&mut self) {
                    assert!(self.0.is_absolute() && self.0.starts_with(std::env::temp_dir()));
                    assert!(
                        self.0
                            .file_name()
                            .unwrap()
                            .to_string_lossy()
                            .starts_with("kinesin-private-")
                    );
                    let _ = std::fs::remove_dir_all(&self.0);
                }
            }
            let _cleanup = Cleanup(base.clone());
            let extra: Vec<String> = std::env::var("KINESIN_TEST_OPERATOR_SID")
                .ok()
                .into_iter()
                .collect();
            let policy = PrivateStatePolicy::current(&extra).unwrap();
            let grants = policy
                .inner
                .permitted
                .iter()
                .map(|sid| format!("(A;OICI;FA;;;{sid})"))
                .collect::<String>();
            let private = descriptor(&format!("O:{}D:P{grants}", current_sid().unwrap()));
            let security = SECURITY_ATTRIBUTES {
                nLength: std::mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
                lpSecurityDescriptor: private.0,
                bInheritHandle: 0,
            };
            let root = base.join("private");
            let wide: Vec<u16> = root.as_os_str().encode_wide().chain(Some(0)).collect();
            // SAFETY: fresh path, terminated UTF-16, live descriptor and attributes.
            assert_ne!(unsafe { CreateDirectoryW(wide.as_ptr(), &security) }, 0);
            std::fs::write(root.join("journal"), b"synthetic").unwrap();
            let report = validate_private_tree(&root, &policy).unwrap();
            assert_eq!(report.checked_entries, 2);
            drop(report);

            let broad = descriptor(&format!(
                "O:{}D:P{grants}(A;;FR;;;WD)",
                current_sid().unwrap()
            ));
            let security = SECURITY_ATTRIBUTES {
                lpSecurityDescriptor: broad.0,
                ..security
            };
            let broad_path = root.join("broad");
            let wide: Vec<u16> = broad_path
                .as_os_str()
                .encode_wide()
                .chain(Some(0))
                .collect();
            // SAFETY: CREATE_NEW prevents modifying existing files; all pointers live.
            let handle = unsafe {
                CreateFileW(
                    wide.as_ptr(),
                    READ_CONTROL,
                    FILE_SHARE_READ | FILE_SHARE_WRITE,
                    &security,
                    CREATE_NEW,
                    FILE_ATTRIBUTE_NORMAL,
                    null_mut(),
                )
            };
            assert_ne!(handle, INVALID_HANDLE_VALUE);
            // SAFETY: successful CreateFileW returned a fresh owned handle.
            drop(unsafe { OwnedHandle::from_raw_handle(handle) });
            assert_eq!(
                validate_private_tree(&root, &policy).unwrap_err(),
                "private_state_untrusted_allow_ace"
            );
            std::fs::remove_file(broad_path).unwrap();

            let outside = base.join("outside-sentinel");
            std::fs::write(&outside, b"outside synthetic sentinel").unwrap();
            std::fs::hard_link(&outside, root.join("hard-link")).unwrap();
            assert_eq!(
                validate_private_tree(&root, &policy).unwrap_err(),
                "private_state_hard_link"
            );
        }
    }
}

#[cfg(unix)]
mod platform {
    use super::*;
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};

    #[derive(Debug)]
    pub struct Policy {
        uid: u32,
    }

    impl Policy {
        pub fn current(extra: &[String]) -> Result<Self, String> {
            if !extra.is_empty() {
                return Err("private_state_windows_sids_on_unix".into());
            }
            // SAFETY: geteuid has no arguments and returns the process credential.
            Ok(Self {
                uid: unsafe { libc::geteuid() },
            })
        }
    }

    pub(super) fn inspect(path: &Path, policy: &Policy, _root: bool) -> Result<Inspected, String> {
        let metadata =
            std::fs::symlink_metadata(path).map_err(|_| "private_state_metadata_failed")?;
        if !metadata.is_dir() && !metadata.is_file() {
            return Err("private_state_special_file".into());
        }
        let file = File::options()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(path)
            .map_err(|_| "private_state_open_failed")?;
        let metadata = file
            .metadata()
            .map_err(|_| "private_state_metadata_failed")?;
        if !metadata.is_dir() && !metadata.is_file() {
            return Err("private_state_special_file".into());
        }
        if metadata.uid() != policy.uid {
            return Err("private_state_untrusted_owner".into());
        }
        if metadata.mode() & 0o077 != 0 {
            return Err("private_state_permissive_mode".into());
        }
        if metadata.is_file() && metadata.nlink() != 1 {
            return Err("private_state_hard_link".into());
        }
        Ok(Inspected {
            file,
            directory: metadata.is_dir(),
            owner: metadata.uid().to_string(),
        })
    }
}

#[cfg(not(any(windows, unix)))]
compile_error!("private state validation requires a supported Windows or Unix platform");

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn relative_paths_fail_before_filesystem_access() {
        let policy = PrivateStatePolicy::current(&[]).unwrap();
        assert_eq!(
            validate_private_tree(&PathBuf::from("state"), &policy).unwrap_err(),
            "private_state_requires_absolute_path"
        );
    }

    #[cfg(unix)]
    #[test]
    fn unix_modes_and_hard_links_are_checked_on_real_files() {
        use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
        let root = std::env::temp_dir().join(format!("kinesin-private-{}", uuid::Uuid::new_v4()));
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&root)
            .unwrap();
        let file = root.join("journal");
        std::fs::write(&file, b"synthetic").unwrap();
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();
        let policy = PrivateStatePolicy::current(&[]).unwrap();
        assert_eq!(
            validate_private_tree(&root, &policy)
                .unwrap()
                .checked_entries,
            2
        );
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert_eq!(
            validate_private_tree(&root, &policy).unwrap_err(),
            "private_state_permissive_mode"
        );
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();
        std::fs::hard_link(&file, root.join("alias")).unwrap();
        assert_eq!(
            validate_private_tree(&root, &policy).unwrap_err(),
            "private_state_hard_link"
        );
        assert!(
            root.starts_with(std::env::temp_dir())
                && root
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("kinesin-private-")
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
