const products = [
    { name: 'Pen', price: 1.50, category: 'Stationery' },
    { name: 'Pencil', price: 0.50, category: 'Stationery' },
    { name: 'Notebook', price: 2.00, category: 'Stationery' },
    { name: 'Eraser', price: 0.75, category: 'Stationery' },
    { name: 'Highlighter', price: 1.00, category: 'Stationery' },
    { name: 'Stapler', price: 3.00, category: 'Office Supplies' }
];

const cart = [];

function renderProducts() {
    const productsDiv = document.getElementById('products');
    productsDiv.innerHTML = '';
    products.forEach(product => {
        const productDiv = document.createElement('div');
        productDiv.className = 'product';
        productDiv.innerHTML = `<h3>${product.name}</h3><p>Price: $${product.price.toFixed(2)}</p><p>Category: ${product.category}</p><button onclick="addToCart('${product.name}')">Add to Cart</button>`;
        productsDiv.appendChild(productDiv);
    });
}

function addToCart(name) {
    const product = products.find(p => p.name === name);
    if (product) {
        cart.push(product);
        renderCart();
    }
}

function renderCart() {
    const cartDiv = document.getElementById('cart');
    cartDiv.innerHTML = '';
    cart.forEach(item => {
        const cartItemDiv = document.createElement('div');
        cartItemDiv.className = 'cart-item';
        cartItemDiv.innerHTML = `<h3>${item.name}</h3><p>Price: $${item.price.toFixed(2)}</p><button onclick="removeFromCart('${item.name}')">Remove</button>`;
        cartDiv.appendChild(cartItemDiv);
    });
    updateTotal();
}

function removeFromCart(name) {
    const index = cart.findIndex(item => item.name === name);
    if (index !== -1) {
        cart.splice(index, 1);
        renderCart();
    }
}

function updateTotal() {
    const total = cart.reduce((acc, item) => acc + item.price, 0);
    const totalDiv = document.getElementById('total');
    totalDiv.innerHTML = `Total: $${total.toFixed(2)}`;
}

function checkout() {
    const name = document.getElementById('name').value;
    const email = document.getElementById('email').value;
    if (name && email) {
        const orderConfirmationDiv = document.getElementById('orderConfirmation');
        orderConfirmationDiv.innerHTML = `<p>Thank you, ${name}! Your order has been confirmed. Please check your email at ${email} for details.</p>`;
        cart = [];
        renderCart();
    }
}

document.getElementById('checkoutForm').addEventListener('submit', (e) => {
    e.preventDefault();
    checkout();
});

renderProducts();