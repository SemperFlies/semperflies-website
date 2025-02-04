'use strict';

(function() {
    class ImgCarousel extends HTMLElement {
        constructor() {
            super();
            this.attachShadow({ mode: "open" });

            this.currentSlideIndex = 0;
            this.autoScroll = this.getAttribute("auto-scroll") === "true";
            this.imageList = (this.getAttribute("images") || "").split(",").map(url => url.trim());
            this.autoScrollInterval = null;
        }

        connectedCallback() {
            this.render();
            if (this.autoScroll) {
                this.startAutoScroll();
            }
        }

        disconnectedCallback() {
            this.stopAutoScroll();
        }

        startAutoScroll() {
            this.autoScrollInterval = setInterval(() => this.nextSlide(), 5000);
        }

        stopAutoScroll() {
            if (this.autoScrollInterval) {
                clearInterval(this.autoScrollInterval);
            }
        }

        nextSlide() {
            this.currentSlideIndex = (this.currentSlideIndex + 1) % this.imageList.length;
            this.updateCarousel();
        }

        previousSlide() {
            this.currentSlideIndex = (this.currentSlideIndex - 1 + this.imageList.length) % this.imageList.length;
            this.updateCarousel();
        }

        updateCarousel() {
            const slides = this.shadowRoot.querySelectorAll(".carousel-slide");
            slides.forEach((slide, index) => {
                slide.style.display = index === this.currentSlideIndex ? "block" : "none";
            });
        }

        render() {
            this.shadowRoot.innerHTML = `
                <style>
                    .carousel-container {
                        position: relative;
                        max-width: 600px;
                        overflow: hidden;
                        text-align: center;
                    }
                    .carousel-slide {
                        display: none;
                        width: 100%;
                    }
                    .carousel-slide img {
                        width: 100%;
                        border-radius: 10px;
                    }
                    .carousel-buttons {
                        position: absolute;
                        top: 50%;
                        width: 100%;
                        display: flex;
                        justify-content: space-between;
                        transform: translateY(-50%);
                    }
                    .carousel-buttons button {
                        background: rgba(0, 0, 0, 0.5);
                        color: white;
                        border: none;
                        padding: 10px;
                        cursor: pointer;
                        font-size: 18px;
                    }
                    .carousel-buttons button:hover {
                        background: rgba(0, 0, 0, 0.8);
                    }
                </style>
                <div class="carousel-container">
                    ${this.imageList
                        .map((imgSrc, index) => `
                            <div class="carousel-slide" style="display: ${index === 0 ? "block" : "none"};">
                                <img src="${imgSrc}" alt="Slide ${index + 1}">
                            </div>
                        `)
                        .join("")}
                    ${this.imageList.length > 1 && !this.autoScroll
                        ? `<div class="carousel-buttons">
                            <button class="prev-btn">&#9665;</button>
                            <button class="next-btn">&#9655;</button>
                           </div>`
                        : ""}
                </div>
            `;

            if (this.imageList.length > 1 && !this.autoScroll) {
                this.shadowRoot.querySelector(".prev-btn").addEventListener("click", () => this.previousSlide());
                this.shadowRoot.querySelector(".next-btn").addEventListener("click", () => this.nextSlide());
            }
        }
    }

    customElements.define('img-carousel', ImgCarousel);
})();
