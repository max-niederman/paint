const { defineConfig } = require("pollen-css/utils");

module.exports = (pollen) => defineConfig({
    output: "./src/styles/pollen.css",
    modules: {
        font: {
            ...pollen.font,
            sans: `'Inter', ${pollen.font.sans}`,
            serif: `'Playfair Display', ${pollen.font.serif}`,
        }
    }
});