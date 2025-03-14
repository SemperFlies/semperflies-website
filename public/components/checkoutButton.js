class CheckoutButton extends HTMLElement {

  constructor() {
    super();
    this.attachShadow({ mode: "open" });
    this.debounceTimeout = null; // Store the timeout ID
    this.debounceDelay = 200; // Delay in milliseconds
    /**
    * @type {string | null}
    */
    this.error = null;
    this.cartEmpty = true;
    this.loading = true;
    this.checkoutUrl = "";
  }

  /**
  * @returns {customerCredentials | null}
  */
  get customerCredentials() {
    const custStor = localStorage.getItem("customer-credentials");  
    if (!custStor) {
      return null;        
    }
    return JSON.parse(custStor);
  }

  /**
  * @returns {ShoppingCart | null}
  */
  get cart() {
      const cartStor = localStorage.getItem("cart");
      if (!cartStor) {
        return null;        
      }
      return JSON.parse(cartStor);
  }

  connectedCallback() {
    this.render();
    this.fetchCheckoutUrl();
    window.addEventListener("updateCheckoutButton", this.handleUpdateCheckoutButton);
  }

  disconnectedCallback() {
      window.removeEventListener("updateCheckoutButton", this.handleUpdateCheckoutButton);
      if (this.debounceTimeout) {
        clearTimeout(this.debounceTimeout); 
      }
  }

  handleUpdateCheckoutButton = () => {
      if (this.debounceTimeout) {
        clearTimeout(this.debounceTimeout); 
      }

      this.debounceTimeout = setTimeout(() => {
        this.fetchCheckoutUrl();
      }, this.debounceDelay);
  }



  render() {
    this.shadowRoot.innerHTML = `
      <style>
        button {
          padding: 10px 20px;
          font-size: 16px;
          border: none;
          cursor: pointer;
          border-radius: 5px;
          background: #007bff;
          color: white;
        }
        .error {
          color: red;
        }
        button:disabled {
          background: grey;
        }
        .spinner {
          width: 24px;
          height: 24px;
          border: 4px solid rgba(255, 255, 255, 0.3);
          border-top: 4px solid white;
          border-radius: 50%;
          animation: spin 1s linear infinite;
        }
        @keyframes spin {
          from { transform: rotate(0deg); }
          to { transform: rotate(360deg); }
        }
      </style>
      ${this.loading ?
        '<div class="spinner"></div>' :
        this.error ? 
        `<p class="error">${this.error}</p>` :
        this.customerCredentials ?
        this.cartEmpty ?
        `<button id="checkout" disabled>Empty Cart</button>` :
        `<button id="checkout">Checkout</button>`
        :
        `<button id="checkout" disabled>No Customer Info</button>`
      }
    `;

    if (!this.loading) {
      this.shadowRoot.getElementById("checkout").addEventListener("click", () => {
        window.location.href = this.checkoutUrl;
      });
    }
  }
  
  async fetchCheckoutUrl() {
    console.log("try to fetch");
    this.loading = true;
    try {
      if (!this.cart || !this.cart.items || Object.keys(this.cart.items).length === 0 || !this.customerCredentials) {
        console.log("do not have necessary info");
        return;
      }
      this.cartEmpty = false;
      /**
      * see `checkout.rs` for the definition of the expected payload
      * @typedef {Object} checkoutEndpointPayload 
      * @property {Object<string, number>} items - A hashmap of product stringIds and their quantities.
      * @property {customerCredentials} customer
      */
      /**
      * @type {checkoutEndpointPayload}
      */
      const itemsPayload = {
        items: Object.fromEntries(
          Object.entries(this.cart.items).map(([id, product]) => [id, product.quantity])
        ),
        customer: this.customerCredentials
      };
      
      const response = await fetch("/stripe/checkout", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(itemsPayload),
      });

      if (!response.ok) {
        /**
        * @typedef ResponseError
        * @field {string} status
        * @field {string} message
        */
        /**  @type {ResponseError} */
        let err = await response.json()
        throw new Error(`Failed: ${err.message}`);
      }

      const data = await response.json();
      if (data.status !== "success" || !data.url) {
        throw new Error("Invalid response from server");
      }
      
      this.checkoutUrl = data.url;
    } catch (error) {
      console.error("Error:", error);
      this.error = `${error}`;
      alert("Something went wrong. Please try again.");
    } finally {
      this.loading = false;
      this.render();
    }
  }
}

customElements.define("checkout-button", CheckoutButton);
