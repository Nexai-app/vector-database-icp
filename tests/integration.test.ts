import { expect, test, describe, it} from "vitest";
import { Actor, CanisterStatus, HttpAgent, Identity } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import {array, random} from 'vectorious';
import {createActor} from '../src/declarations/vector_database_backend'
import canisterIds from '../.dfx/local/canister_ids.json';
import {Ed25519KeyIdentity} from '@dfinity/identity';
import {Secp256k1KeyIdentity} from '@dfinity/identity-secp256k1';
import * as fs from 'fs';
import exp from "vectorious/dist/core/exp";

import * as crypto from 'node:crypto';
import { AccessControl } from "../src/declarations/vector_database_backend/vector_database_backend.did";

const EMBEDDING_SIZE = 768;

function genRandomEmbedding() : number[] {
    let embedding = random(EMBEDDING_SIZE);
    let arr = embedding.toArray();
    return arr;
}


function getIdentityBySeed(seedfile: string): Identity {
    if(!fs.existsSync(seedfile)) {
        console.log("generating randomly")
        return Secp256k1KeyIdentity.generate() 
    }
    let seed = fs.readFileSync(seedfile).toString();
    let identity = Secp256k1KeyIdentity.fromSeedPhrase(seed);
    console.log(identity.getPrincipal().toText());
    return identity;
}

let owner = getIdentityBySeed("keys/owner.seed");
let comp1 = getIdentityBySeed("keys/comp1.seed");
let comp2 = getIdentityBySeed("keys/comp2.seed");
let manager = getIdentityBySeed("keys/manager.seed");
let user = getIdentityBySeed("keys/user.seed");

test("should gen embedding", () => {
    let arr = genRandomEmbedding();
    expect(arr.length).eq(EMBEDDING_SIZE);
})

describe("access control should work", async () => {
    let vdb = createActor(canisterIds.vector_database_backend.local, {
        agentOptions: {
            host: "http://127.0.0.1:4943",
            fetch,
            identity: owner
        }
    });
    let vdb_user = createActor(canisterIds.vector_database_backend.local, {
        agentOptions: {
            host: "http://127.0.0.1:4943",
            fetch,
            identity: user
        }
    });
    let vdb_manager = createActor(canisterIds.vector_database_backend.local, {
        agentOptions: {
            host: "http://127.0.0.1:4943",
            fetch,
            identity: manager
        }
    });

    it("should add manager", async () => {
        let result = await vdb.add_manager(manager.getPrincipal()) ;
        expect(result).eq(true);
    });

    it("should not allow unprivileged user add manager or add allow list", async () => {
        expect(async () => await vdb_user.add_manager(user.getPrincipal())).rejects.toThrowError("Not owner");
        expect(async () => await vdb_user.add_accesser(user.getPrincipal())).rejects.toThrowError("Not manager");
    })

    it("should not allow manager to add another manager", async () => {
        expect(async () => await vdb_manager.add_manager(manager.getPrincipal())).rejects.toThrowError("Not owner");
    })

    it("should allow manage to manage accessers", async () => {
        let res = await vdb_manager.add_accesser(user.getPrincipal());
        expect(res).eq(true);
    });

    let comp_id = -1;
    it("should allow user to register and manage their own vdb", async () => {
        let res = await vdb_user.register("Description of the company") as {Ok: number};
        expect('Ok' in res).eq(true);
        comp_id = res.Ok;
    })

    it("should allow user to insert keys and values", async () => {
        let keys = [genRandomEmbedding(), genRandomEmbedding()];
        let values = ["aaa", "bbb"];
        let append_res = await vdb_user.append_keys_values(comp_id, keys, values) as {Ok: null};
        expect('Ok' in append_res).eq(true)
    })

    it("should allow user build index",async () => {
        let build_res = await vdb_user.build_index(comp_id) as {Ok: null};
        expect('Ok' in build_res).eq(true)
    })

    it("should remove user from access list", async () => {
        let res = await vdb_manager.remove_accesser(user.getPrincipal());
        expect(res).eq(true);
    })

    it("should not able to access", async () => {
        let res = await vdb_user.build_index(comp_id) as {Err: string};
        expect(res.Err).eq("caller not owner of company or not manager");
    })

    let vdb_comp1 = createActor(canisterIds.vector_database_backend.local, {
        agentOptions: {
            host: "http://127.0.0.1:4943",
            fetch,
            identity: comp1
        }
    });
    let vdb_comp2 = createActor(canisterIds.vector_database_backend.local, {
        agentOptions: {
            host: "http://127.0.0.1:4943",
            fetch,
            identity: comp2
        }
    });
    let comp1_id = -1;
    let comp2_id = -1;
    it("registers company 1", async () => {
        let res = await vdb_manager.add_accesser(comp1.getPrincipal());
        expect(res).eq(true);
        let res1 = await vdb_comp1.register("comp1") as {Ok: number};
        expect('Ok' in res1).eq(true);
        comp1_id = res1.Ok;
    })
    it("registers company 2", async () => {
        let res = await vdb_manager.add_accesser(comp2.getPrincipal());
        expect(res).eq(true);
        let res1 = await vdb_comp2.register("comp2") as {Ok: number};
        expect('Ok' in res1).eq(true);
        comp2_id = res1.Ok;
    })

    it("does not allow cross access", async () => {
        let res = await vdb_comp1.build_index(comp2_id) as {Err: string};
        expect(res.Err).eq("caller not owner of company or not manager");
        let res1 = await vdb_comp2.build_index(comp1_id) as {Err: string};
        expect(res1.Err).eq("caller not owner of company or not manager");
    })

    it("disable access list to allow everyone to register", async () => {
        let res = await vdb.set_acl_enabled(false) as {'Ok': null};
        expect('Ok' in res).eq(true);
    })

    it("should allow anyone to register", async () => {
        let vdb_anyone = createActor(canisterIds.vector_database_backend.local, {
            agentOptions: {
                host: "http://127.0.0.1:4943",
                fetch,
                // random one
                identity: Ed25519KeyIdentity.generate()
            }
        });

        let res = await vdb_anyone.register("anyone can register") as {Ok: number};
        expect('Ok' in res).eq(true);
    })
})



describe("vector database should work", async () => {
    let identity = Ed25519KeyIdentity.generate();
    let vdb = createActor(canisterIds.vector_database_backend.local, {
        agentOptions: {
            host: "http://127.0.0.1:4943",
            fetch,
            identity: identity
        }
    });

    let a = genRandomEmbedding();
    a[0] = 1;
    let b = genRandomEmbedding();
    b[0] = 0.5;
    let c = genRandomEmbedding();
    c[0] = 0;
    let embds = [a, b, c];
    let values = [
        "question 1?",
        "question 4?",
        "question 6?",
    ];

    let company_id: number = -1;
    it("allow user to access", async () => {
        let vdb_manager = createActor(canisterIds.vector_database_backend.local, {
            agentOptions: {
                host: "http://127.0.0.1:4943",
                fetch,
                identity: manager
            }
        });
        let res = await vdb_manager.add_accesser(identity.getPrincipal());
        expect(res).eq(true);
    })

    it("should register a company", async () => {
        const description = "Simple Company description";
        let result = await vdb.register(description); 
        expect('Ok' in result).eq(true);
        result = result as {'Ok': number};
        company_id = result.Ok;
        expect(company_id >= 0).eq(true);
    })

    it("should accept embeddings", async () => {
        const result = await vdb.append_keys_values(company_id, embds, values);
        expect('Ok' in result).eq(true);
    })

    it("should build index", async () => {
        const result = await vdb.build_index(company_id);
        expect('Ok' in result).eq(true);
    })

    it("should return a same value corresponding to embedding", async () => {
        // embedding, limit
        let result = await vdb.get_similar(company_id, a, 1);
        expect('Ok' in result).eq(true);
        result = result as {'Ok': Array<[number, string]>};
        const vs = result.Ok;
        expect(vs.length).eq(1);
        console.log(vs)
        console.log(values)
        expect(vs[0][1]).eq(values[0]);
    })
})

test("states should be there", async () => {
    let vdb = createActor(canisterIds.vector_database_backend.local, {
        agentOptions: {
            host: "http://127.0.0.1:4943",
            fetch,
        }
    });

    let states = await vdb.states();
    expect(states.length).eq(1);
    states = states as [AccessControl];
    let state = states[0];
    console.log(state);
})