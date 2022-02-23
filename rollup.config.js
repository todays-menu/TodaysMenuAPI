import resolve from '@rollup/plugin-node-resolve';
import graphql from '@rollup/plugin-graphql';
import commonjs from '@rollup/plugin-commonjs';
import json from '@rollup/plugin-json';

export default {
    input: 'index.js',
    output: {
        file: 'dist/index.js',
    },
    plugins: [
        resolve({
            preferBuiltins: true
        }),
        graphql(),
        commonjs(),
        json(),
    ],
    external: [
        '@kanru/biscuit-wasm',
        'apollo-datasource',
        'graphql-upload',
        'apollo-server-core',
        'mongodb',
        'depd',
        'base64-arraybuffer'
    ]
}