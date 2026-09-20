let cart = [];

try {
  const storedCart = JSON.parse(localStorage.getItem('cart'));
  cart = storedCart.filter(item => products.some(product => product.id === item.id) && Number.isInteger(item.quantity) && item.quantity >= 1 && item.quantity <= 99);
} catch (e) {
  // Ignore errors and start with an empty cart
}

function addToCart(id) {
  const existingItem = cart.find(item => item.id === id);
  if (existingItem) {
    existingItem.quantity++;
  } else {
    cart.push({ id, quantity: 1 });
  }
  saveCart();
}

function saveCart() {
  localStorage.setItem('cart', JSON.stringify(cart));
  renderCart();
}

function renderCart() {
  const cartContainer = document.getElementById('cart-items');
  cartContainer.innerHTML = '';

  if (cart.length === 0) {
    cartContainer.innerHTML = '<p>Your cart is empty.</p><div id="cart-total">Total: $0.00</div>'
    return;
  }

  let total = 0;

  cart.forEach(item => {
    const product = products.find(p => p.id === item.id);
    if (product) {
      const itemElement = document.createElement('div');
      itemElement.innerHTML = `<strong>${product.name}</strong> x ${item.quantity} = $${(product.price * item.quantity / 100).toFixed(2)}`;

      const increaseButton = document.createElement('button');
      increaseButton.textContent = 'Increase';
      increaseButton.addEventListener('click', () => {
        item.quantity = Math.min(item.quantity + 1, 99);
        saveCart();
      });

      const decreaseButton = document.createElement('button');
      decreaseButton.textContent = 'Decrease';
      decreaseButton.addEventListener('click', () => {
        item.quantity -= 1;
        if (item.quantity === 0) cart = cart.filter(i => i.id !== item.id);
        saveCart();
      });

      const removeButton = document.createElement('button');
      removeButton.textContent = 'Remove';
      removeButton.addEventListener('click', () => {
        cart = cart.filter(i => i.id !== item.id);
        saveCart();
      });

      itemElement.appendChild(increaseButton);
      itemElement.appendChild(decreaseButton);
      itemElement.appendChild(removeButton);

      cartContainer.appendChild(itemElement);

      total += product.price * item.quantity;
    }
  });

  const totalElement = document.createElement('div');
  totalElement.innerHTML = `<strong>Total: $${(total / 100).toFixed(2)}</strong>`;
  cartContainer.appendChild(totalElement);
}renderCart();
