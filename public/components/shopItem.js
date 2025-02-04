'use strict';

(function() {
  class ShopItem extends HTMLElement {
    constructor() {
      super();
      this.attachShadow({ mode: 'open' });
    }

    connectedCallback() {
      this.render();
    }

    /**
     * Retrieves the item's data from its attributes.
     * @typedef {Object} ShopItemData
     * @property {string} id - The item's ID.
     * @property {string} name - The name of the item.
     * @property {number} price - The price of the item.
     * @property {string[]} imgs - An array of image URLs for the item.
     * @property {string|null} description - The description of the item.
     * @returns {ShopItemData}
     */
    get itemData() {
      return {
        id: this.getAttribute('id'),
        name: this.getAttribute('name'),
        price: parseFloat(this.getAttribute('price') || 0),
        imgs: this.getAttribute('imgs') ? JSON.parse(this.getAttribute('imgs')) : [],
        description: this.getAttribute('description')
      };
    }

    addToCart() {
      const cartItems = JSON.parse(localStorage.getItem("cart-items") || `{}`);
      const { id, name, price, imgs, description } = this.itemData;

      if (id in cartItems) {
        cartItems[id].quantity += 1;
      } else {
        cartItems[id] = { name, price, imgs, description, quantity: 1 };
      }

      localStorage.setItem("cart-items", JSON.stringify(cartItems));

      // Dispatch an event to notify the shopping cart
      window.dispatchEvent(new Event("storage"));
    }

    /**
    * @param {string[]} imgs
    */
    renderCarousel(imgs) {
      let currentIndex = 0;

      const carouselContainer = document.createElement("div");
      carouselContainer.classList.add("carousel-container");
      const prevNextButtons = document.createElement("div");
      prevNextButtons.classList.add("prev-next-buttons");

      const imgElement = document.createElement("img");
      imgElement.src = imgs[currentIndex];
      imgElement.alt = "Product Image";

      carouselContainer.appendChild(imgElement);
      if (imgs.length > 1) { 
          const prevButton = document.createElement("button");
          prevButton.classList.add("material-symbols-outlined");
          prevButton.textContent = "chevron_left";
          prevButton.addEventListener("click", () => {
            currentIndex = (currentIndex - 1 + imgs.length) % imgs.length;
            imgElement.src = imgs[currentIndex];
          });

          const nextButton = document.createElement("button");
          nextButton.classList.add("material-symbols-outlined");
          nextButton.textContent = "chevron_right";
          nextButton.addEventListener("click", () => {
            currentIndex = (currentIndex + 1) % imgs.length;
            imgElement.src = imgs[currentIndex];
          });
          prevNextButtons.appendChild(prevButton);
          prevNextButtons.appendChild(nextButton);
          carouselContainer.appendChild(prevNextButtons);
     }      

      return carouselContainer;
    }

    render() {
      const { name, price, imgs, description } = this.itemData;

      const container = document.createElement("div");
      container.classList.add("shop-item");

      const title = document.createElement("h3");
      title.textContent = name;

      const desc = document.createElement("p");
      desc.textContent = description;

      const priceTag = document.createElement("p");
      priceTag.textContent = `$${price.toFixed(2)}`;
      priceTag.classList.add("price");

      const addButton = document.createElement("button");
      addButton.textContent = "Add to Cart";
      addButton.classList.add("add-to-cart");
      addButton.addEventListener("click", () => this.addToCart());

      container.appendChild(title);
      container.appendChild(this.renderCarousel(imgs));
      container.appendChild(desc);
      container.appendChild(priceTag);
      container.appendChild(addButton);

      const style = document.createElement("style");
      style.textContent = `
        @import url('https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@20..48,100..700,0..1,-50..200');
        .shop-item {
          border: 1px solid #ddd;
          padding: 10px;
          border-radius: 5px;
          text-align: center;
          max-width: 200px;
          display: flex;
          flex-direction: column;
          align-items: center;
        }
        .carousel-container {
          display: flex;
          flex-direction: column;
          justify-content: center;
          align-items: center;
          margin: 10px 0;
        }
        .carousel-container img {
          max-width: 200px;
          object-fit: cover;
          border-radius: 5px;
        }
        .prev-next-buttons {
          display: flex;
          flex-direction: row;
          justify-content: center;
        }
        .price {
          font-weight: bold;
          color: green;
        }
        .add-to-cart {
          background-color: #007bff;
          color: white;
          border: none;
          padding: 8px;
          cursor: pointer;
          border-radius: 5px;
          margin-top: 5px;
        }
        .add-to-cart:hover {
          background-color: #0056b3;
        }
        button {
          margin: 5px;
          padding: 5px 10px;
        }
      `;

      this.shadowRoot.appendChild(style);
      this.shadowRoot.appendChild(container);
    }
  }

  customElements.define('shop-item', ShopItem);
})();
