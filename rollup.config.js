import resolve from '@rollup/plugin-node-resolve';
import graphql from '@rollup/plugin-graphql';

export default {
    input: 'index.js',
    output: {
        file: 'dist/index.js',
    },
    plugins: [resolve(), graphql()],
    external: [/node_modules/]
}