//! Atomic model-capacity grants with round-robin selection among ready owners.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::{Arc, Mutex};
use tokio::sync::{OwnedSemaphorePermit, Semaphore, oneshot};

struct Request {
    id: u64,
    owner: String,
    backend: String,
    reply: oneshot::Sender<ModelPermit>,
}

struct State {
    next: u64,
    last_owner: Option<String>,
    requests: VecDeque<Request>,
}

pub struct ModelDispatcher {
    global: Arc<Semaphore>,
    backends: BTreeMap<String, Arc<Semaphore>>,
    max_waiters: usize,
    state: Mutex<State>,
}

pub struct ModelPermit {
    global: Option<OwnedSemaphorePermit>,
    backend: Option<OwnedSemaphorePermit>,
    dispatcher: Arc<ModelDispatcher>,
}

impl Drop for ModelPermit {
    fn drop(&mut self) {
        // Release actual resources before considering the next ready owner.
        self.backend.take();
        self.global.take();
        self.dispatcher.dispatch();
    }
}

struct Waiting {
    id: u64,
    dispatcher: Arc<ModelDispatcher>,
}

impl Drop for Waiting {
    fn drop(&mut self) {
        self.dispatcher
            .state
            .lock()
            .expect("model dispatch mutex")
            .requests
            .retain(|request| request.id != self.id);
        self.dispatcher.dispatch();
    }
}

impl ModelDispatcher {
    pub fn new(
        global_slots: usize,
        backends: BTreeMap<String, Arc<Semaphore>>,
        max_waiters: usize,
    ) -> Result<Arc<Self>, String> {
        if global_slots == 0
            || global_slots > Semaphore::MAX_PERMITS
            || max_waiters == 0
            || backends.is_empty()
        {
            return Err("invalid_model_capacity".into());
        }
        Ok(Arc::new(Self {
            global: Arc::new(Semaphore::new(global_slots)),
            backends,
            max_waiters,
            state: Mutex::new(State {
                next: 0,
                last_owner: None,
                requests: VecDeque::new(),
            }),
        }))
    }

    /// The caller applies its queue deadline/cancellation to this future.
    /// Dropping the future removes its waiter or releases an already sent grant.
    pub async fn acquire(
        self: &Arc<Self>,
        owner: &str,
        backend: &str,
    ) -> Result<ModelPermit, String> {
        if owner.is_empty() || owner.len() > 64 || !self.backends.contains_key(backend) {
            return Err("invalid_model_dispatch_identity".into());
        }
        let (reply, receiver) = oneshot::channel();
        let id = {
            let mut state = self.state.lock().expect("model dispatch mutex");
            if state.requests.len() >= self.max_waiters {
                return Err("model_dispatch_overloaded".into());
            }
            let id = state.next;
            state.next = state
                .next
                .checked_add(1)
                .ok_or("model_dispatch_counter_exhausted")?;
            state.requests.push_back(Request {
                id,
                owner: owner.into(),
                backend: backend.into(),
                reply,
            });
            id
        };
        let _waiting = Waiting {
            id,
            dispatcher: self.clone(),
        };
        self.dispatch();
        receiver.await.map_err(|_| "model_dispatch_closed".into())
    }

    pub fn waiting(&self) -> usize {
        self.state
            .lock()
            .expect("model dispatch mutex")
            .requests
            .len()
    }

    fn dispatch(self: &Arc<Self>) {
        let grants = {
            let mut state = self.state.lock().expect("model dispatch mutex");
            state.requests.retain(|request| !request.reply.is_closed());
            let mut grants = Vec::new();
            while self.global.available_permits() > 0 {
                let ready: BTreeSet<_> = state
                    .requests
                    .iter()
                    .filter(|request| self.backends[&request.backend].available_permits() > 0)
                    .map(|request| request.owner.as_str())
                    .collect();
                let owner = ready
                    .iter()
                    .find(|owner| {
                        state
                            .last_owner
                            .as_ref()
                            .is_none_or(|last| **owner > last.as_str())
                    })
                    .or_else(|| ready.first())
                    .map(|owner| (*owner).to_owned());
                let Some(owner) = owner else { break };
                let position = state
                    .requests
                    .iter()
                    .position(|request| {
                        request.owner == owner
                            && self.backends[&request.backend].available_permits() > 0
                    })
                    .expect("ready model request");
                // Both try-acquisitions occur under one dispatcher lock: no
                // request holds one scarce permit while awaiting another.
                let Ok(global) = self.global.clone().try_acquire_owned() else {
                    break;
                };
                let backend_name = &state.requests[position].backend;
                let Ok(backend) = self.backends[backend_name].clone().try_acquire_owned() else {
                    drop(global);
                    break;
                };
                let request = state
                    .requests
                    .remove(position)
                    .expect("selected model request");
                state.last_owner = Some(owner);
                grants.push((
                    request.reply,
                    ModelPermit {
                        global: Some(global),
                        backend: Some(backend),
                        dispatcher: self.clone(),
                    },
                ));
            }
            grants
        };
        // A disconnected receiver releases its permit, which dispatches again.
        // Never drop that permit while holding the dispatcher mutex.
        for (reply, permit) in grants {
            let _ = reply.send(permit);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::timeout;

    fn dispatcher(global: usize) -> Arc<ModelDispatcher> {
        ModelDispatcher::new(
            global,
            BTreeMap::from([
                ("one".into(), Arc::new(Semaphore::new(1))),
                ("two".into(), Arc::new(Semaphore::new(1))),
            ]),
            4,
        )
        .unwrap()
    }

    #[tokio::test]
    async fn ready_owners_rotate_and_cancelled_waiters_release_every_reservation() {
        let gate = dispatcher(1);
        let first = gate.acquire("alice", "one").await.unwrap();
        let mut again = Box::pin(gate.acquire("alice", "one"));
        let mut other = Box::pin(gate.acquire("bob", "one"));
        assert!(
            timeout(Duration::from_millis(10), &mut again)
                .await
                .is_err()
        );
        assert!(
            timeout(Duration::from_millis(10), &mut other)
                .await
                .is_err()
        );
        assert_eq!(gate.waiting(), 2);
        drop(first);
        let bob = timeout(Duration::from_secs(1), other)
            .await
            .unwrap()
            .unwrap();
        assert!(
            timeout(Duration::from_millis(10), &mut again)
                .await
                .is_err()
        );
        drop(again);
        assert_eq!(gate.waiting(), 0);
        drop(bob);
        assert_eq!(gate.global.available_permits(), 1);
        assert_eq!(gate.backends["one"].available_permits(), 1);
    }

    #[tokio::test]
    async fn busy_backend_does_not_hold_capacity_needed_by_another_backend() {
        let gate = dispatcher(2);
        let first = gate.acquire("alice", "one").await.unwrap();
        let mut blocked = Box::pin(gate.acquire("alice", "one"));
        assert!(
            timeout(Duration::from_millis(10), &mut blocked)
                .await
                .is_err()
        );
        let second = timeout(Duration::from_secs(1), gate.acquire("bob", "two"))
            .await
            .unwrap()
            .unwrap();
        assert_eq!(gate.global.available_permits(), 0);
        drop(blocked);
        drop(first);
        drop(second);
        assert_eq!(gate.global.available_permits(), 2);
    }
}
