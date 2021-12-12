import { DataSource } from "apollo-datasource";
import { Collection, ObjectId } from "mongodb";

export default class Dishes extends DataSource {
    /**
     * @param {Collection} collection MongoDB collection
     */
    constructor(collection) {
        super();
        this.collection = collection;
    }

    async getDishes() {
        return (await this.collection.find().toArray())
            .map(doc => Object.assign({ id: doc._id }, doc));
    }

    async addNewDishes(dishes) {
        let names = dishes.map(d => d.name);
        let duplicates = await this.collection.countDocuments({ name: { $in: names } });
        if (duplicates > 0) {
            return { success: false, message: "found duplicated dishes" };
        }
        await this.collection.insertMany(dishes);
        return { success: true }
    }

    async updateDishes(dishes) {
        let bulkOp = this.collection.initializeUnorderedBulkOp();
        dishes.forEach(dish => {
            dish = Object.assign({ _id: new ObjectId(dish.id) }, dish);
            delete dish.id;
            bulkOp.find({ _id: { $eq: dish._id } }).replaceOne(dish);
        });
        let result = await bulkOp.execute();
        return { success: true, message: `modified ${result.nModified} dishes` };
    }
}