"use strict";

const products = {
  pencil: { name: "Copper Pencil", priceCents: 200 },
  notebook: { name: "Field Notebook", priceCents: 500 }
};
let itemCount = 0;
let totalCents = 0;

document.querySelectorAll("button[data-product]").forEach((button) => {
  button.addEventListener("click", () => {
    const product = products[button.dataset.product];
    itemCount += 1;
    totalCents = product.priceCents;
    const entry = document.createElement("li");
    entry.textContent = product.name;
    document.getElementById("cart-items").append(entry);
    document.getElementById("item-count").textContent = String(itemCount);
    document.getElementById("total").textContent = "$" + (totalCents / 100).toFixed(2);
  });
});

document.getElementById("search").addEventListener("input", (event) => {
  const query = event.target.value.trim().toLowerCase();
  document.querySelectorAll(".product").forEach((product) => {
    product.hidden = !product.dataset.name.toLowerCase().includes(query);
  });
});
