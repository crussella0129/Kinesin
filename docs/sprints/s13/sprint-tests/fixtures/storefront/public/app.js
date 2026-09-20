const products = [
  { id: 'desk-lamp', name: 'Desk Lamp', price: 2500, category: 'lighting' },
  { id: 'desk-organizer', name: 'Desk Organizer', price: 1500, category: 'lighting' },
  { id: 'bookshelf', name: 'Bookshelf', price: 3000, category: 'paper' },
  { id: 'pen-set', name: 'Pen Set', price: 500, category: 'paper' },
  { id: 'chair', name: 'Chair', price: 10000, category: 'furniture' },
  { id: 'desk-mat', name: 'Desk Mat', price: 2000, category: 'furniture' }
];

function renderProducts() {
  const searchInput = document.getElementById('search-input').value.trim().toLowerCase();
  const categorySelect = document.getElementById('category-select').value;
  const productGrid = document.getElementById('product-grid');
  productGrid.innerHTML = '';

  const filteredProducts = products.filter(product =>
    product.name.toLowerCase().includes(searchInput) &&
    (categorySelect === 'all' || product.category === categorySelect)
  );

  if (filteredProducts.length === 0) {
    productGrid.innerHTML = '<p>No products found</p>';
  } else {
    filteredProducts.forEach(product => {
      const productCard = document.createElement('div');
      productCard.className = 'product-card';
      productCard.innerHTML = `<p>${product.name} - $${product.price / 100}</p>
      <button >Add to Cart</button>`;
      productGrid.appendChild(productCard);
productCard.querySelector('button').addEventListener('click', () => addToCart(product.id));
    });
  }
}

document.getElementById('search-input').addEventListener('input', renderProducts);
document.getElementById('category-select').addEventListener('change', renderProducts);

renderProducts();