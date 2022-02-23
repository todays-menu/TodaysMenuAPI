import biscuit from "@kanru/biscuit-wasm";

export default class Authorizer {
    constructor(publicKey) {
        this.publicKey = publicKey;
    }

    authorize(rawToken) {
        const token = biscuit.Biscuit.from_base64(rawToken, this.publicKey);
        const authorizer = token.authorizer();
        authorizer.add_policy('allow if user($user_id), role($user_id, "admin")');
        authorizer.add_policy('deny if true');
        const accepted_policy = authorizer.authorize();
        return true;
    }
}