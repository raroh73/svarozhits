/** @type {import('tailwindcss').Config} */
module.exports = {
    content: ["templates/src/*.html"],
    theme: {
        extend: {},
    },
    plugins: [
        require('@tailwindcss/forms'),
    ],
}
