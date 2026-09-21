"use strict";

const catalog = {
  pad: { name: "Grid Pad", cents: 400 },
  tape: { name: "Paper Tape", cents: 300 }
};
const basket = {};

function renderBasket() {
  const rows = document.getElementById("rows");
  rows.replaceChildren();
  let units = 0;
  let cents = 0;
  for (const [id, quantity] of Object.entries(basket)) {
    units += quantity;
    cents += catalog[id].cents * quantity;
    const row = document.createElement("li");
    const label = document.createElement("span");
    label.textContent = catalog[id].name + " × " + quantity;
    const remove = document.createElement("button");
    remove.type = "button";
    remove.textContent = "Remove " + catalog[id].name;
    remove.addEventListener("click", () => {
      basket[id] -= 1;
      if (basket[id] === 0) delete basket[id];
      renderBasket();
    });
    row.append(label, remove);
    rows.append(row);
  }
  document.getElementById("count").textContent = String(units);
  document.getElementById("amount").textContent = "$" + (cents / 100).toFixed(2);
}

document.querySelectorAll("button[data-add]").forEach((button) => {
  button.addEventListener("click", () => {
    const id = button.dataset.add;
    basket[id] = (basket[id] || 0) + 1;
    renderBasket();
  });
});

document.getElementById("filter").addEventListener("input", (event) => {
  const query = event.target.value.trim().toLowerCase();
  document.querySelectorAll(".supply").forEach((product) => {
    product.hidden = !product.dataset.name.toLowerCase().includes(query);
  });
});
