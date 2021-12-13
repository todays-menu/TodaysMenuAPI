import { GraphQLUpload } from "graphql-upload";

export const resolvers = {
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
    RecipeIngredient: {
        ingredient: async (parent, _, { dataSources: { ingredients } }) =>
            await ingredients.findOneByName(parent.description),
    }
};