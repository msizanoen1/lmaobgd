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
  ]
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
        }
      ]
    },
    resolve: {
      extensions: [".js", ".ts", ".tsx"]
    },
    output: {
      filename: "bundle.js",
      path: path.resolve(__dirname, "dist")
    },
    devtool: "source-map"
  };
};
