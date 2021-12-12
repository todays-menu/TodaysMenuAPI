import { DataSource } from "apollo-datasource";
import { Collection } from "mongodb";
import { randomBytes } from "crypto";

export default class Shares extends DataSource {
    /**
     * @param {Collection} collection MongoDB collection
     */
    constructor(collection) {
        super();
        this.collection = collection;
    }

    async addNewShare(shareableWeeklyMenu) {
        let key = Buffer.from(randomBytes(6)).toString('base64');
        await this.collection.insertOne({ _id: key, shareableWeeklyMenu });
        return { success: true, key }
    }
}