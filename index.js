import { ApolloServer, gql } from 'apollo-server-express';
import { ApolloServerPluginDrainHttpServer } from 'apollo-server-core';
import { GraphQLUpload, graphqlUploadExpress } from 'graphql-upload';
import { MongoClient } from 'mongodb';
import express from 'express';
import http from 'http';
import Dishes from './src/data-sources/Dishes.js';
import Ingredients from './src/data-sources/Ingredients.js';
import Shares from './src/data-sources/Shares.js';

const typeDefs = gql`
    type Dish {
        id: ID!
        name: String!
        meal: Meal!
        ingredient: [RecipeIngredient]!
        meat: Float
        vegetable: Float
        seafood: Float
        spice: Boolean
        cook_time: Int
        iCook: String
        amount: Int
        solo(is: Boolean): Boolean
        soup: Boolean
        style: String
    }

    enum Meal {
        lunch
        dinner
    }

    type RecipeIngredient {
        description: String!
        quantity: String!
    }

    type Ingredient {
        id: ID!
        name: String!
        category: String!
    }

    input ShareDishInput {
        photo: String
        name: String!
        cook_time: Int!
    }

    input ShareDayMenuInput {
        date: String!
        lunch: [ShareDishInput!]
        dinner: [ShareDishInput!]
    }

    input ShareableWeeklyMenuInput {
        menus: [ShareDayMenuInput!]!
    }

    type ShareableWeeklyMenu {
        success: Boolean!
        message: String
        key: String
    }

    input NewDishInput {
        name: String!
        meal: Meal!
        ingredient: [NewDishRecipeIngredientInput]!
        meat: Float
        vegetable: Float
        seafood: Float
        spice: Boolean
        cook_time: Int
        iCook: String
        amount: Int
        solo: Boolean
        soup: Boolean
        style: String
    }

    input UpdateDishInput {
        id: ID!
        name: String!
        meal: Meal!
        ingredient: [UpdateDishRecipeIngredientInput]!
        meat: Float
        vegetable: Float
        seafood: Float
        spice: Boolean
        cook_time: Int
        iCook: String
        amount: Int
        solo: Boolean
        soup: Boolean
        style: String
    }

    input NewDishRecipeIngredientInput {
        description: String!
        quantity: String!
    }

    input UpdateDishRecipeIngredientInput {
        description: String!
        quantity: String!
    }

    input NewIngredientInput {
        name: String!
        category: String!
    }

    input UpdateIngredientInput {
        id: ID!
        name: String!
        category: String!
    }

    type MutationResult {
        success: Boolean!
        message: String
    }

    scalar Upload
    type File {
        filename: String!
        mimetype: String!
        encoding: String!
    }

    type Query {
        dishes: [Dish]
        ingredients: [Ingredient]
    }

    type Mutation {
        shareWeeklyMenu(menu: ShareableWeeklyMenuInput): ShareableWeeklyMenu
        uploadDishPhoto(file: Upload!): File!
        addNewDishes(dishes: [NewDishInput]): MutationResult
        updateDishes(dishes: [UpdateDishInput]): MutationResult
        addNewIngredients(ingredients: [NewIngredientInput]): MutationResult
        updateIngredients(ingredients: [UpdateIngredientInput]): MutationResult
    }
`;

const resolvers = {
    Upload: GraphQLUpload,
    Query: {
        dishes: async (_, __, { dataSources: { dishes } }) => await dishes.getDishes(),
        ingredients: async (_, __, { dataSources: { ingredients } }) => await ingredients.getIngredients(),
    },
    Mutation: {
        shareWeeklyMenu: async (_, { menu }, { dataSources: { share: ds } }) =>
            await ds.addNewShare(menu),
        uploadDishPhoto: async (_, { file }) => {
            const { createReadStream, filename, mimetype, encoding } = await file;
            const stream = createReadStream();
            console.log(stream);
            return { filename, mimetype, encoding };
        },
        addNewDishes: async (_, { dishes }, { dataSources: { dishes: ds } }) =>
            await ds.addNewDishes(dishes),
        updateDishes: async (_, { dishes }, { dataSources: { dishes: ds } }) =>
            await ds.updateDishes(dishes),
        addNewIngredients: async (_, { ingredients }, { dataSources: { ingredients: ds } }) =>
            await ds.addNewIngredients(ingredients),
        updateIngredients: async (_, { ingredients }, { dataSources: { ingredients: ds } }) =>
            await ds.updateIngredients(ingredients),
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
    const db = client.db("test");
    const server = new ApolloServer({
        typeDefs,
        resolvers,
        plugins: [ApolloServerPluginDrainHttpServer({ httpServer })],
        dataSources: () => ({
            share: new Shares(db.collection('shares')),
            dishes: new Dishes(db.collection('dishes')),
            ingredients: new Ingredients(db.collection('ingredients')),
        })
    });
    await server.start();
    app.use(graphqlUploadExpress());
    server.applyMiddleware({ app });
    await new Promise(resolve => httpServer.listen({ port: 8080 }, resolve));
    console.log(`🚀  Server ready at http://localhost:8080${server.graphqlPath}`);
}

startApolloServer(typeDefs, resolvers);