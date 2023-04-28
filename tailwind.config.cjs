/** @type {import('tailwindcss').Config} */
module.exports = {
	content: {
		files: ["*.html", "./src/**/*.rs"],
	},
	theme: {
		extend: {
			colors: {
				gray: {
					100: "#dbdbdb",
					300: "#595959",
					600: "#303030",
					700: "#2a2a2a",
					800: "#202020",
					900: "#161616",
				},
			},
		},
	},
	plugins: [],
};
