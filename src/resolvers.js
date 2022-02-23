import { GraphQLUpload } from "graphql-upload";

export const resolvers = {
    Upload: GraphQLUpload,
    Query: {
        dishes: async (_, __, { dataSources: { dishes } }) => await dishes.getDishes(),
        ingredients: async (_, __, { dataSources: { ingredients } }) => await ingredients.getIngredients(),
        shareableMenu: async (_, { key }, { dataSources: { shares } }) => await shares.findOneByKey(key),
    },
    Mutation: {
        shareMenu: async (_, { menu }, { dataSources: { shares: ds } }) =>
            await ds.addNewShareableMenu(menu),
        addNewDishes: async (_, { dishes }, {  authorizer, token, dataSources: { dishes: ds } }) => {
            try {
                authorizer.authorize(token);
            } catch {
                return { success: false, message: 'permission denied' };
            }
            return await ds.addNewDishes(dishes);
        },
        updateDishes: async (_, { dishes }, {  authorizer, token, dataSources: { dishes: ds } }) => {
            try {
                authorizer.authorize(token);
            } catch {
                return { success: false, message: 'permission denied' };
            }
            return await ds.updateDishes(dishes);
        },
        addNewIngredients: async (_, { ingredients }, { authorizer, token, dataSources: { ingredients: ds } }) => {
            try {
                authorizer.authorize(token);
            } catch {
                return { success: false, message: 'permission denied' };
            }
            return await ds.addNewIngredients(ingredients);
        },
        loginUser: async (_, { userId, password }, { dataSources: { authenticator: auth } }) =>
            await auth.authenticate(userId, password),
    },
    RecipeIngredient: {
        ingredient: async (recipeIngredient, _, { dataSources: { ingredients } }) =>
            await ingredients.findOneByName(recipeIngredient.name),
    }
};