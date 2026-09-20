// Global variables for cart
let cart = [];
let cartTotal = 0;

// Function to update the cart display
function updateCart() {
    const cartItems = document.getElementById('cart-items');
    cartItems.innerHTML = '';
    cartTotal = 0;

    cart.forEach((item, index) => {
        const li = document.createElement('li');
        li.innerHTML = `${item.name} - $${item.price.toFixed(2)} <button onclick="removeFromCart(${index})">Remove</button>`;
        cartItems.appendChild(li);
        cartTotal += item.price * item.quantity;
    });

    document.getElementById('cart-total').textContent = `$${cartTotal.toFixed(2)}`;
}

// Function to add an item to the cart
function addToCart(product) {
    const existingItem = cart.find(item => item.name === product.name);
    if (existingItem) {
        existingItem.quantity += 1;
    } else {
        cart.push({ name: product.name, price: product.price, quantity: 1 });
    }
    updateCart();
}

// Function to remove an item from the cart
function removeFromCart(index) {
    cart.splice(index, 1);
    updateCart();
}

// Function to filter products by category
function filterProducts(category) {
    const products = document.getElementById('products');
    products.innerHTML = '';

    if (category === 'all') {
        productsProducts.forEach(product => {
            const productDiv = document.createElement('div');
            productDiv.className = 'product';
            productDiv.innerHTML = `<h3>${product.name}</h3><p>$${product.price.toFixed(2)}</p><button onclick="addToCart(${JSON.stringify(product)})">Add to Cart</button>`;
            products.appendChild(productDiv);
        });
    } else {
        productsProducts.forEach(product => {
            if (product.category === category) {
                const productDiv = document.createElement('div');
                productDiv.className = 'product';
                productDiv.innerHTML = `<h3>${product.name}</h3><p>$${product.price.toFixed(2)}</p><button onclick="addToCart(${JSON.stringify(product)})">Add to Cart</button>`;
                products.appendChild(productDiv);
            }
        });
    }
}

// Function to handle checkout
function checkout() {
    const name = prompt('Enter your name:');
    const email = prompt('Enter your email:');

    if (name && email) {
        alert(`Thank you, ${name}! Your order has been confirmed. We'll send a confirmation email to ${email} shortly.`);
        cart = [];
        cartTotal = 0;
        updateCart();
    }
}

// Initialize products
const productsProducts = [
    { name: 'Notebook', price: 12.99, category: 'stationery' },
    { name: 'Pens', price: 8.49, category: 'stationery' },
    { name: 'Sticky Notes', price: 3.99, category: 'stationery' },
    { name: 'Desk Calendar', price: 15.99, category: 'office' },
    { name: 'File Folders', price: 7.99, category: 'office' },
    { name: 'Desk Lamp', price: 29.99, category: 'office' }
];

// Initial product display
filterProducts('all');
