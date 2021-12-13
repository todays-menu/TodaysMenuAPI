import { ApolloServer } from 'apollo-server-express';
import { ApolloServerPluginDrainHttpServer } from 'apollo-server-core';
import { GraphQLUpload, graphqlUploadExpress } from 'graphql-upload';
import { MongoClient } from 'mongodb';
import express from 'express';
import http from 'http';
import Dishes from './src/data-sources/dishes.js';
import Ingredients from './src/data-sources/ingredients.js';
import Shares from './src/data-sources/shares.js';
import typeDefs from './src/schema.graphql';


const resolvers = {
    Upload: GraphQLUpload,
    Query: {
        dishes: async (_, __, { dataSources: { dishes } }) => await dishes.getDishes(),
        ingredients: async (_, __, { dataSources: { ingredients } }) => await ingredients.getIngredients(),
    },
    Mutation: {
        shareMenu: async (_, { menu }, { dataSources: { share: ds } }) =>
            await ds.addNewShare(menu),
        addNewDishes: async (_, { dishes }, { dataSources: { dishes: ds } }) =>
            await ds.addNewDishes(dishes),
        updateDishes: async (_, { dishes }, { dataSources: { dishes: ds } }) =>
            await ds.updateDishes(dishes),
        addNewIngredients: async (_, { ingredients }, { dataSources: { ingredients: ds } }) =>
            await ds.addNewIngredients(ingredients),
    },
};

async function startApolloServer(typeDefs, resolvers) {
    const app = express();
    const httpServer = http.createServer(app);
    const dbusername = encodeURIComponent(process.env.DB_USERNAME);
    const dbpassword = encodeURIComponent(process.env.DB_PASSWORD);
    const clusterUrl = process.env.DB_URL;
    const client = new MongoClient(`mongodb://${dbusername}:${dbpassword}@${clusterUrl}/`);
    await client.connect();
    const db = client.db(process.env.DB_NAME);
    const server = new ApolloServer({
        typeDefs,
        resolvers,
        plugins: [ApolloServerPluginDrainHttpServer({ httpServer })],
        dataSources: () => ({
            share: new Shares(db.collection('shares')),
            dishes: new Dishes(db.collection('dishes'), process.env.PHOTOS_PATH),
            ingredients: new Ingredients(db.collection('ingredients')),
        })
    });
    await server.start();
    app.use(graphqlUploadExpress());
    app.use('/photos', express.static(process.env.PHOTOS_PATH));
    server.applyMiddleware({ app });
    await new Promise(resolve => httpServer.listen({ port: 8080 }, resolve));
    console.log(`🚀  Server ready at http://localhost:8080${server.graphqlPath}`);
}

startApolloServer(typeDefs, resolvers);