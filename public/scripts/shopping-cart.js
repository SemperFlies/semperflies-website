const states = {
  "Alabama": "AL",
  "Alaska": "AK",
  "Arizona": "AZ",
  "Arkansas": "AR",
  "California": "CA",
  "Colorado": "CO",
  "Connecticut": "CT",
  "Delaware": "DE",
  "Florida": "FL",
  "Georgia": "GA",
  "Hawaii": "HI",
  "Idaho": "ID",
  "Illinois": "IL",
  "Indiana": "IN",
  "Iowa": "IA",
  "Kansas": "KS",
  "Kentucky": "KY",
  "Louisiana": "LA",
  "Maine": "ME",
  "Maryland": "MD",
  "Massachusetts": "MA",
  "Michigan": "MI",
  "Minnesota": "MN",
  "Mississippi": "MS",
  "Missouri": "MO",
  "Montana": "MT",
  "Nebraska": "NE",
  "Nevada": "NV",
  "New Hampshire": "NH",
  "New Jersey": "NJ",
  "New Mexico": "NM",
  "New York": "NY",
  "North Carolina": "NC",
  "North Dakota": "ND",
  "Ohio": "OH",
  "Oklahoma": "OK",
  "Oregon": "OR",
  "Pennsylvania": "PA",
  "Rhode Island": "RI",
  "South Carolina": "SC",
  "South Dakota": "SD",
  "Tennessee": "TN",
  "Texas": "TX",
  "Utah": "UT",
  "Vermont": "VT",
  "Virginia": "VA",
  "Washington": "WA",
  "West Virginia": "WV",
  "Wisconsin": "WI",
  "Wyoming": "WY"
};


/**
 * Gets the customer credentials from localStorage.
 * @returns {customerCredentials | null} The customer's credentials if they exist.
 */
function getCustomerCredentials() {
  const storedCredentials = localStorage.getItem("customer-credentials");
  return storedCredentials ? JSON.parse(storedCredentials) : null;
}

/**
 * Handles form submission and stores the data in localStorage.
 * @param {Event} event The form submission event.
 */
function handleFormSubmit(event) {
  event.preventDefault();
  const formData = new FormData(event.target);
  const customerData = {};
  formData.forEach((value, key) => {
    customerData[key] = value;
  });

  localStorage.setItem("customer-credentials", JSON.stringify(customerData));
  window.dispatchEvent(new Event("updateCheckoutButton"));
}

function renderCart() {
  /**
  * @type {ShoppingCart}
  */
  const cartData = JSON.parse(localStorage.getItem("cart") || "{}");
  const cartContainer = document.getElementById("cart");
  const totalPriceElement = document.getElementById("cart-total");

  cartContainer.innerHTML = "";
  cartContainer.classList.add("items-container");

  let totalPrice = 0;

  if (Object.keys(cartData).length === 0) {
    cartContainer.innerHTML = "<p>Your cart is empty.</p>";
    totalPriceElement.textContent = "Total: $0.00";
    return;
  }

  Object.entries(cartData.items).forEach(([id, item]) => {
    const itemTotal = (item.price * item.quantity) / 100;
    totalPrice += itemTotal;

    const itemDiv = document.createElement("div");
    const formattedPrice = (item.price / 100).toFixed(2);
    itemDiv.classList.add("cart-item");
    itemDiv.innerHTML = `
      <p><strong>${item.name}</strong></p>
      <img src=${item.images[0]}></img>
      <p>Each: $${formattedPrice}</p>
      <p>Quantity: ${item.quantity}</p>
      <p><strong>Total: $${itemTotal.toFixed(2)}</strong></p>
      <button class="small" onclick="addOne('${id}')">Add One</button>
      <button class="small" onclick="removeOne('${id}')">Remove One</button>
      <button class="small" onclick="removeAll('${id}')">Remove All</button>
    `;
    cartContainer.appendChild(itemDiv);
  });

  totalPriceElement.textContent = `Total: $${totalPrice.toFixed(2)}`;

  // Customer credentials handling
  const customerForm = document.getElementById("customer-form");
  const formValues = getCustomerCredentials() || {
    name: "",
    email: "",
    line1: "",
    line2: "",
    zipCode: "",
    state: "",
    city: ""
  };

  customerForm.querySelector("[name='name']").value = formValues.name;
  customerForm.querySelector("[name='email']").value = formValues.email;
  customerForm.querySelector("[name='line1']").value = formValues.line1;
  customerForm.querySelector("[name='line2']").value = formValues.line2;
  customerForm.querySelector("[name='zipCode']").value = formValues.zipCode;
  customerForm.querySelector("[name='state']").value = formValues.state;
  customerForm.querySelector("[name='city']").value = formValues.city;

  const stateSelect = customerForm.querySelector("[name='state']");
  Object.entries(states).forEach(([state, abbreviation]) => {
    const option = document.createElement("option");
    option.value = abbreviation;
    option.textContent = `${state} (${abbreviation})`;
    if (formValues.state === abbreviation) {
      option.selected = true;
    }
    stateSelect.appendChild(option);
  });

  window.dispatchEvent(new Event("updateCheckoutButton"));
  customerForm.addEventListener("submit", handleFormSubmit);
}


/**
* @param {string} itemId - Id of the item to add to cart
*/ 
function addOne(itemId) {
  /**
  * @type {ShoppingCart}
  */
  const cartData = JSON.parse(localStorage.getItem("cart") || "{}");
  if (cartData.items[itemId]) {
    cartData.items[itemId].quantity += 1;
    localStorage.setItem("cart", JSON.stringify(cartData));
    renderCart();
  }
}

/**
* @param {string} itemId - Id of the item to remove from the cart
*/ 
function removeOne(itemId) {
  /**
  * @type {ShoppingCart}
  */
  const cartData = JSON.parse(localStorage.getItem("cart") || "{}");
  if (cartData.items[itemId]) {
    if (cartData.items[itemId].quantity > 1) {
      cartData.items[itemId].quantity -= 1;
    } else {
      delete cartData.items[itemId];
    }
    localStorage.setItem("cart", JSON.stringify(cartData));
    renderCart();
  }
}

/**
* @param {string} itemId - Id of the item to remove from the cart
*/ 
function removeAll(itemId) {
  /**
  * @type {ShoppingCart}
  */
  const cartData = JSON.parse(localStorage.getItem("cart") || "{}");
  delete cartData.items[itemId];
  localStorage.setItem("cart", JSON.stringify(cartData));
  renderCart();
}

renderCart();
