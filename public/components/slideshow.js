"use strict";

/**
 * Identical to the `Image` type in `components/carousel.rs`
 * @typedef {Object} SlideshowImage
 * @property {string} src - The source URL of the image.
 * @property {string} alt - The alt text for the image.
 * @property {string} subtitle - The subtitle or caption for the image.
 */

class ImageSlideshow extends HTMLElement {
    constructor() {
        super();
        this.attachShadow({ mode: 'open' });
        this.currentIndex = 0;
        this.interval = null;
    }

    /** @returns {SlideshowImage[]} */
    get images() {
        let attr = this.getAttribute("data");
        /** @type {SlideshowImage[]} */
        let obj = JSON.parse(attr);
        return obj;
    }

   
    connectedCallback() {
        this.render();
        this.startSlideshow();
    }

    disconnectedCallback() {
        clearInterval(this.interval);
    }

    startSlideshow() {
        clearInterval(this.interval);
        this.interval = setInterval(() => this.nextImage(), 8000);
    }

    nextImage() {
        this.currentIndex = (this.currentIndex + 1) % this.images.length;
        this.updateImage();
    }

    updateImage() {
        if (!this.images.length) return;
        const image = this.images[this.currentIndex];
        this.shadowRoot.querySelector('img').src = image.src;
        this.shadowRoot.querySelector('img').alt = image.alt;
        this.shadowRoot.querySelector('figcaption').textContent = image.subtitle;
    }

    render() {
        this.shadowRoot.innerHTML = `
            <style>
                :host {
                    display: flex;
                    justify-content: center;
                    align-items: center;
                    overflow: hidden;
                    max-width: 100%;
                    border: 2px double var(--primary-light);
                    padding: 0.5rem;
                    background-color: var(--primary-dark);
                }
                figure {
                    margin: 0;
                    text-align: center;
                }
                img {
                    max-width: 100%;
                    max-height: 100vh;
                    object-fit: contain;
                    display: block;
                }
                figcaption {
                    font-size: 1rem;
                    margin-top: 8px;
                    color: var(--primary-light);
                }
            </style>
            <figure>
                <img src="" alt="">
                <figcaption></figcaption>
            </figure>
        `;
        this.updateImage();
    }
}

customElements.define('image-slideshow', ImageSlideshow);

