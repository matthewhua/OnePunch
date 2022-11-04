const path = require('path')

const utils = require('./utils')
const webpack = require('webpack')
const nodeExternals = require('webpack-node-externals')

const { CleanWebpackPlgin } = require('clean-webpack-plugin')

const webpackconfig = {
    target: 'node',
    entry: {
        server: path.join(utils.)
    }
}