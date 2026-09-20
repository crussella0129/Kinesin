document.getElementById('checkout-form').addEventListener('submit', function(event) {
  event.preventDefault();
  const name = document.getElementById('name').value.trim();
  const email = document.getElementById('email').value;
  const emailInput = document.getElementById('email');
  if (cart.length === 0) {
    confirmation.textContent = 'Your cart is empty.';
  } else if (name === '' || !emailInput.checkValidity()) {
    confirmation.textContent = 'Invalid name or email.';
  } else {
    const total = cart.reduce((acc, item) => {
      const product = products.find(p => p.id === item.id);
      return acc + product.price * item.quantity;
    }, 0);
    confirmation.textContent = `Order confirmed, ${name}! Total: $${(total / 100).toFixed(2)}`;
    cart = [];
    saveCart();
  }
});