const path = require('path')
const

const webpackConfig = {
    target: 'node',
    mode: "development",
    entry: {
        server: path.join(__dirname, 'src/index.js')
    },
    output: {
        path: './dist'
    },

}