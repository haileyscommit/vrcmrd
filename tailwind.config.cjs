module.exports = {
  content: ["./index.html", "./src/**/*.{ts,tsx,js,jsx}"],
  // Safelist for classes that may be built dynamically or appear in expressions
  // safelist: [
  //   'bg-white',
  //   'bg-gray-50',
  //   'bg-gray-800',
  //   'bg-gray-900',
  //   'dark:bg-transparent',
  //   'text-gray-500',
  //   'text-gray-900',
  //   'text-gray-100',
  //   'text-xs',
  //   'text-sm',
  //   'text-lg',
  //   'font-semibold',
  //   'px-2',
  //   'py-1',
  //   { pattern: /bg-(gray|amber)-(50|100|200|300|400|500|600|700|800|900)/ },
  //   { pattern: /text-(gray|amber)-(50|100|200|300|400|500|600|700|800|900)/ },
  //   { pattern: /p(x|y)-\d/ },
  //   { pattern: /w-\d/ },
  //   { pattern: /h-\d/ }
  // ]
  theme: {
    extend: {
      colors: {
        amber: {
          400: '#f59e0b'
        }
      }
    }
  },
  plugins: []
};
