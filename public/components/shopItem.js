'use strict';

(function() {
  class ShopItem extends HTMLElement {
    constructor() {
      super();
      this.attachShadow({ mode: 'open' });
    }

    connectedCallback() {
      console.log(this.product);
      this.render();
    }

    /**
     * This component expects to receive a product JSON object as a string in the 'product' attribute
     * @returns {Product} - the parsed product
     */
    get product() {
      let str = this.getAttribute("product");
      let product  = JSON.parse(str);
      return product;
    }

    addToCart() {
      const cartItems = JSON.parse(localStorage.getItem("cart-items") || `{}`);
      const { id }  = this.product;

      if (id in cartItems) {
        cartItems[id] += 1;
      } else {
        cartItems[id] = 1;
      }

      localStorage.setItem("cart-items", JSON.stringify(cartItems));

      // Dispatch an event to notify the shopping cart
      window.dispatchEvent(new Event("storage"));
    }



    render() {
      const { name, default_price, images, description } = this.product;

      const container = document.createElement("div");
      container.classList.add("shop-item");

      const title = document.createElement("h3");
      title.textContent = name;

      const desc = document.createElement("p");
      desc.textContent = description;

      const priceTag = document.createElement("p");

      // unit_amount is in cents
      const formattedPrice = (default_price.unit_amount / 100).toFixed(2);
      priceTag.textContent = `$${formattedPrice}`;
      priceTag.classList.add("price");

      const addButton = document.createElement("button");
      addButton.textContent = "Add to Cart";
      addButton.classList.add("add-to-cart");
      addButton.addEventListener("click", () => this.addToCart());

      container.appendChild(title);
      container.appendChild(this.renderCarousel(images));
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
  }


  customElements.define('shop-item', ShopItem);
})();
