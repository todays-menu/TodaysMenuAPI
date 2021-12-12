import { DataSource } from "apollo-datasource";
import { Collection } from "mongodb";
import { randomBytes } from "crypto";
import { createWriteStream } from "fs";

export default class Shares extends DataSource {
    /**
     * @param {Collection} collection MongoDB collection
     */
    constructor(collection, base_path) {
        super();
        this.base_path = base_path;
        this.collection = collection;
    }

    async uploadDishPhoto(file) {
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
        await this.collection.insertOne({ key, filename, mimetype, encoding });
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