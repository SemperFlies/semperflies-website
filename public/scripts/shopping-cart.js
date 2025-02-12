
function renderCart() {
  /** @type {CartData} */
  const cartData = JSON.parse(localStorage.getItem("cart-items") || "{}");
  const cartContainer = document.getElementById("cart-items");
  const totalPriceElement = document.getElementById("cart-total");

  cartContainer.innerHTML = "";
  cartContainer.classList.add("items-container");

  let totalPrice = 0;

  if (Object.keys(cartData).length === 0) {
    cartContainer.innerHTML = "<p>Your cart is empty.</p>";
    totalPriceElement.textContent = "Total: $0.00";
    return;
  }

  Object.entries(cartData).forEach(([id, item]) => {
    const itemTotal = item.price * item.quantity;
    totalPrice += itemTotal;

    const itemDiv = document.createElement("div");
    itemDiv.classList.add("cart-item");
    itemDiv.innerHTML = `
      <p><strong>${item.name}</strong></p>
      <img src=${item.imgs[0]}></img>
      <p>Each: $${item.price.toFixed(2)}</p>
      <p>Quantity: ${item.quantity}</p>
      <p><strong>Total: $${itemTotal.toFixed(2)}</strong></p>
      <button onclick="addOne('${id}')">Add One</button>
      <button onclick="removeOne('${id}')">Remove One</button>
      <button onclick="removeAll('${id}')">Remove All</button>
      <hr>
    `;

    cartContainer.appendChild(itemDiv);
  });

  totalPriceElement.textContent = `Total: $${totalPrice.toFixed(2)}`;
}

function addOne(itemId) {
  const cartData = JSON.parse(localStorage.getItem("cart-items") || "{}");
  if (cartData[itemId]) {
    cartData[itemId].quantity += 1;
    localStorage.setItem("cart-items", JSON.stringify(cartData));
    renderCart();
  }
}

function removeOne(itemId) {
  const cartData = JSON.parse(localStorage.getItem("cart-items") || "{}");
  if (cartData[itemId]) {
    if (cartData[itemId].quantity > 1) {
      cartData[itemId].quantity -= 1;
    } else {
      delete cartData[itemId];
    }
    localStorage.setItem("cart-items", JSON.stringify(cartData));
    renderCart();
  }
}

function removeAll(itemId) {
  const cartData = JSON.parse(localStorage.getItem("cart-items") || "{}");
  delete cartData[itemId];
  localStorage.setItem("cart-items", JSON.stringify(cartData));
  renderCart();
}

renderCart();

