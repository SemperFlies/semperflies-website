'use strict';

(function() {
    class ShoppingCart extends HTMLElement {
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

        get items() {
            return JSON.parse(localStorage.getItem("cart-items") || `{}`);
        }

        get itemCount() {
            return Object.values(this.items).reduce((acc, item) => acc + (item.quantity || 1), 0);
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
                window.location.href = "/shopping_cart";
            });

            const style = document.createElement("style");
            style.textContent = `
                @import url('https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200');
                .cart-container {
                    position: relative;
                    cursor: pointer;
                    display: inline-flex;
                    align-items: center;
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

    customElements.define('shopping-cart', ShoppingCart);
})();
