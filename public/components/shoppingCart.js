'use strict';

(function() {
    class ShoppingCartComponent extends HTMLElement {
        constructor() {
            super();
            this.attachShadow({ mode: 'open' });
        }

        connectedCallback() {
            this.render();
            window.addEventListener("storage", () => this.updateCount());
        }

        disconnectedCallback() {
            window.removeEventListener("storage", () => this.updateCount());
        }
        /**
        * @returns {ShoppingCart} the shopping cart in localStorage
        */
        get cart() {
           let cart = localStorage.getItem("cart");
           if (!cart || !JSON.parse(cart).items) {
                const emptyCart = { items: {} }; 
                localStorage.setItem("cart", JSON.stringify(emptyCart));
                return emptyCart;
            }
            return JSON.parse(cart);
        }

        get itemCount() {
            return Object.values(this.cart.items).reduce((acc, item) => acc + (item.quantity || 1), 0);
        }

        updateCount() {
            const countSpan = this.shadowRoot.querySelector(".cart-count");
            if (countSpan) {
                countSpan.textContent = this.itemCount;
                countSpan.style.display = this.itemCount > 0 ? "inline" : "none";
            }
        }
        
        render() {

            const container = document.createElement("div");
            container.classList.add("cart-container");

            const cartSpan = document.createElement("span");
            cartSpan.classList.add("material-symbols-outlined");
            cartSpan.textContent = "shopping_cart";


            const countSpan = document.createElement("span");
            countSpan.classList.add("cart-count");
            countSpan.textContent = this.itemCount;
            countSpan.style.display = this.itemCount > 0 ? "inline" : "none";

            container.appendChild(cartSpan);
            container.appendChild(countSpan);

            container.addEventListener("click", () => {
              let url = "/shopping_cart"; 
              window.location.assign(url);
            });

            const style = document.createElement("style");
            style.textContent = `
                @import url('https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200');
                .cart-container {
                    position: relative;
                    cursor: pointer;
                    display: inline-flex;
                    align-items: center;
                    border-radius: 8px;
                    padding: 5px;
                    border: 1px solid var(--primary-light);
                    background-color: var(--primary-dark);
                    transition: all ease-in-out 200ms;
                }
                .cart-container:hover {
                    background-color: var(--primary-light);
                    color: var(--primary-dark);
                }
                .cart-count {
                    background: red;
                    color: white;
                    font-size: 12px;
                    font-weight: bold;
                    border-radius: 50%;
                    width: 18px;
                    height: 18px;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    position: absolute;
                    text-align: center;
                    top: -5px;
                    right: -10px;
                }
            `;

            this.shadowRoot.appendChild(style);
            this.shadowRoot.appendChild(container);
        }
    }

    customElements.define('shopping-cart', ShoppingCartComponent);
})();
