let products = [
    { name: 'Pen', price: 1.50, category: 'Stationery' },
    { name: 'Pencil', price: 0.50, category: 'Stationery' },
    { name: 'Eraser', price: 0.25, category: 'Stationery' },
    { name: 'Notebook', price: 2.00, category: 'Stationery' },
    { name: 'Highlighter', price: 1.00, category: 'Stationery' },
    { name: 'Stapler', price: 3.00, category: 'Stationery' }
];

let cart = [];

function addToCart(product) {
    cart.push(product);
}

function removeFromCart(index) {
    cart.splice(index, 1);
}

function calculateTotal() {
    return cart.reduce((total, item) => total + item.price, 0);
}

function displayCart() {
    let cartElement = document.getElementById('cart');
    cartElement.innerHTML = '';
    cart.forEach((item, index) => {
        let itemElement = document.createElement('div');
        itemElement.innerHTML = `${item.name} - $${item.price} <button onclick='removeFromCart(${index})'>Remove</button>`;
        cartElement.appendChild(itemElement);
    });
    let totalElement = document.getElementById('total');
    totalElement.innerHTML = `Total: $${calculateTotal()}`;
}

function searchProducts(query) {
    let searchResults = products.filter(product => product.name.toLowerCase().includes(query.toLowerCase()));
    let searchElement = document.getElementById('searchResults');
    searchElement.innerHTML = '';
    searchResults.forEach(product => {
        let productElement = document.createElement('div');
        productElement.innerHTML = `${product.name} - $${product.price} - ${product.category}`;
        searchElement.appendChild(productElement);
    });
}

function filterProducts(category) {
    let filteredProducts = products.filter(product => product.category === category);
    let productsElement = document.getElementById('products');
    productsElement.innerHTML = '';
    filteredProducts.forEach(product => {
        let productElement = document.createElement('div');
        productElement.innerHTML = `${product.name} - $${product.price} - ${product.category}`;
        productsElement.appendChild(productElement);
    });
}

window.onload = () => {
    displayCart();
    searchProducts('');
    filterProducts('Stationery');
};