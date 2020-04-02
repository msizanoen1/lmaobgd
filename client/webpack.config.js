const path = require("path");

const babelOptions = {
  presets: [
    [
      "@babel/env",
      {
        useBuiltIns: "usage",
        corejs: 3
      }
    ]
  ],
  plugins: ["@babel/plugin-transform-runtime"]
};

module.exports = function(env, argv) {
  const babel = env != null && env.babel == "true";
  return {
    entry: "./src/index.ts",
    module: {
      rules: [
        {
          test: /\.tsx?$/,
          use: (() => {
            let use = [];
            if (babel) {
              use.push({
                loader: "babel-loader",
                options: babelOptions
              });
            }
            use.push({
              loader: "ts-loader"
            });
            return use;
          })(),
          exclude: /node_modules/
        },
        {
          test: /\.jsx?$/,
          use: babel
            ? [
                {
                  loader: "babel-loader",
                  options: babelOptions
                }
              ]
            : [],
          exclude: /node_modules/
        }
      ]
    },
    resolve: {
      extensions: [".js", ".ts", ".tsx"]
    },
    output: {
      filename: "bundle.js",
      path: path.resolve(__dirname, "dist"),
      library: "LmaoBGD",
      libraryTarget: "umd",
      libraryExport: "default"
    },
    devtool: "source-map"
  };
};
