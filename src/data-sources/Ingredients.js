import { DataSource } from "apollo-datasource";
import { Collection, ObjectId } from "mongodb";

export default class Ingredients extends DataSource {
    /**
     * @param {Collection} collection MongoDB collection
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

    async updateIngredients(ingredients) {
        let bulkOp = this.collection.initializeUnorderedBulkOp();
        ingredients.forEach(ingredient => {
            ingredient = Object.assign({ _id: new ObjectId(ingredient._id) }, ingredient);
            delete ingredient.id;
            bulkOp.find({ _id: { $eq: ingredient._id } }).replaceOne(ingredient);
        });
        let result = await bulkOp.execute();
        return { success: true, message: `modified ${result.nModified} ingredients` };
    }
}