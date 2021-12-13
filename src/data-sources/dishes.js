import { DataSource } from "apollo-datasource";
import { randomBytes } from "crypto";
import { createWriteStream } from "fs";

export default class Dishes extends DataSource {
    /**
     * @param {import('mongodb').Collection} collection MongoDB collection
     */
    constructor(collection, base_path) {
        super();
        this.collection = collection;
        this.base_path = base_path;
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
        let bulkOp = this.collection.initializeUnorderedBulkOp();
        await Promise.all(dishes.map(async (dish) => {
            if (dish.photo) {
                dish.photo = await this.uploadDishPhoto(dish.name, dish.photo);
                if (!dish.photo.success) {
                    return { success: false, message: `photo upload failed. ${dish.photo.message}` };
                }
                delete dish.photo.success;
            }
            bulkOp.insert(dish);
        }));
        let result = await bulkOp.execute();
        return { success: true, message: `inserted ${result.nInserted} dishes` };
    }

    async updateDishes(dishes) {
        let bulkOp = this.collection.initializeUnorderedBulkOp();
        dishes.forEach(dish => {
            bulkOp.find({ name: { $eq: dish.name } }).updateOne({ $set: dish });
        });
        let result = await bulkOp.execute();
        return { success: true, message: `modified ${result.nModified} dishes` };
    }

    async uploadDishPhoto(name, file) {
        const key = Buffer.from(randomBytes(16)).toString('hex');
        const { createReadStream, mimetype, encoding } = await file;
        const stream = createReadStream();
        const extension = this.mimeExtension(mimetype);
        if (!extension) {
            return { success: false, message: `unsupported mimetype ${mimetype}` };
        }
        const filename = `${key}.${extension}`;
        const path = `${this.base_path}/${filename}`
        const out = createWriteStream(path);
        stream.pipe(out);
        stream.on('error', (e) => {
            out.destroy(e);
            return { success: false };
        })
        await new Promise(resolve => out.on('finish', resolve));
        await this.collection.insertOne({ key, name, filename, mimetype, encoding });
        return { success: true, filename, mimetype, encoding };
    }

    mimeExtension(mimetype) {
        if (mimetype === 'image/jpeg') {
            return 'jpg';
        }
        if (mimetype === 'image/png') {
            return 'png';
        }
        return null;
    }
}