import { DataSource } from "apollo-datasource";
import biscuit from "@kanru/biscuit-wasm";
import { Sha256 } from "@aws-crypto/sha256-js";
import base64 from "base64-arraybuffer";

export default class Authenticator extends DataSource {
    /**
     * 
     * @param {import('mongodb').Collection} collection MongoDB collection
     */
    constructor(rootKey, collection) {
        super()
        this.collection = collection;
        this.rootKey = rootKey;
    }

    async authenticate(userId, password) {
        const user = await this.collection.findOne({ userId: { $eq: userId } });

        if (user == null) {
            // wait some time before return to prevent brute force attack
            await new Promise(resolve => setTimeout(resolve, 5000));
            return { success: false, message: 'user not found' };
        }

        const salt = user.salt;
        const hash = new Sha256();
        hash.update(base64.decode(password));
        hash.update(salt);
        const key = base64.encode(await hash.digest());

        if (user.key !== key) {
            return { success: false, message: 'incorrect password' };
        }

        // Authenticated
        let builder = biscuit.Biscuit.builder();
        builder.add_authority_fact(`user("${userId}")`);
        user.roles.forEach(role => {
            builder.add_authority_fact(`role("${userId}", "${role}")`);
        });
        let token = builder.build(this.rootKey).to_base64();

        return { success: true, message: 'authenticated', token };
    }
}