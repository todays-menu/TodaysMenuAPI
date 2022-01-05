import { DataSource } from "apollo-datasource";
import { randomBytes } from "crypto";

export default class Shares extends DataSource {
    /**
     * @param {import('mongodb').Collection} collection MongoDB collection
     */
    constructor(collection) {
        super();
        this.collection = collection;
    }

    async addNewShareableMenu(shareableMenu) {
        let key = Buffer.from(randomBytes(6)).toString('base64url');
        await this.collection.insertOne({ _id: key, payload: shareableMenu.payload });
        return { success: true, key }
    }

    async findOneByKey(key) {
        let item = await this.collection.findOne({ _id: { $eq: key } });
        if (item == null) {
            return null;
        }
        return { key, payload: item.payload };
    }
}