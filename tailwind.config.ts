import type { Config } from "tailwindcss";

const config: Config = {
  content: ["./index.html", "./src/**/*.{vue,js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        primary: {
          DEFAULT: "#7EC850",
          dark: "#5BA030",
          light: "#9FE070",
        },
        surface: {
          DEFAULT: "rgba(255,255,255,0.08)",
          hover: "rgba(255,255,255,0.12)",
          active: "rgba(255,255,255,0.16)",
          blur: "rgba(255,255,255,0.05)",
        },
        danger: "#E05050",
        warning: "#E0C050",
        info: "#5090E0",
        success: "#7EC850",
      },
      fontFamily: {
        display: ['"Press Start 2P"', '"Mojangles"', "monospace"],
        body: ['"Determination Mono"', "monospace"],
      },
      borderRadius: {
        pixel: "4px",
      },
      borderWidth: {
        pixel: "2px",
      },
      backdropBlur: {
        glass: "20px",
      },
      boxShadow: {
        pixel: "4px 4px 0px 0px rgba(0,0,0,0.3)",
        "pixel-sm": "2px 2px 0px 0px rgba(0,0,0,0.3)",
        "pixel-lg": "6px 6px 0px 0px rgba(0,0,0,0.25)",
        glow: "0 0 20px rgba(126,200,80,0.3)",
        "glow-strong": "0 0 40px rgba(126,200,80,0.5)",
      },
      animation: {
        "pixel-bounce": "pixelBounce 0.6s ease-in-out infinite alternate",
        "fade-in": "fadeIn 0.3s ease-out",
        "slide-up": "slideUp 0.3s ease-out",
        "glow-pulse": "glowPulse 2s ease-in-out infinite alternate",
      },
      keyframes: {
        pixelBounce: {
          "0%": { transform: "translateY(0px)" },
          "100%": { transform: "translateY(-4px)" },
        },
        fadeIn: {
          "0%": { opacity: "0" },
          "100%": { opacity: "1" },
        },
        slideUp: {
          "0%": { opacity: "0", transform: "translateY(10px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
        glowPulse: {
          "0%": { boxShadow: "0 0 20px rgba(126,200,80,0.2)" },
          "100%": { boxShadow: "0 0 40px rgba(126,200,80,0.5)" },
        },
      },
      spacing: {
        pixel: "2px",
        "pixel-2": "4px",
        "pixel-3": "6px",
        "pixel-4": "8px",
      },
    },
  },
  plugins: [],
};

export default config;
