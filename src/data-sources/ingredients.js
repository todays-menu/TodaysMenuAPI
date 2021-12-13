import { DataSource } from "apollo-datasource";

export default class Ingredients extends DataSource {
    /**
     * @param {import('mongodb').Collection} collection MongoDB collection
     */
    constructor(collection) {
        super();
        this.collection = collection;
    }

    async getIngredients() {
        return (await this.collection.find().toArray())
            .map(doc => Object.assign({ id: doc._id }, doc));
    }

    async addNewIngredients(ingredients) {
        let names = ingredients.map(d => d.name);
        let duplicates = await this.collection.countDocuments({ name: { $in: names } });
        if (duplicates > 0) {
            return { success: false, message: "found duplicated ingredients" };
        }
        await this.collection.insertMany(ingredients);
        return { success: true }
    }
}