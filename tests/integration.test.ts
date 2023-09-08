import { expect, test, describe, it} from "vitest";
import { Actor, CanisterStatus, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import {array, random} from 'vectorious';
import {createActor} from '../src/declarations/vector_database_backend'
import canisterIds from '../.dfx/local/canister_ids.json';
import {ECDSAKeyIdentity} from '@dfinity/identity';

const EMBEDDING_SIZE = 768;

function genRandomEmbedding() : number[] {
    let embedding = random(EMBEDDING_SIZE);
    let arr = embedding.toArray();
    return arr;
}

test("should gen embedding", () => {
    let arr = genRandomEmbedding();
    expect(arr.length).eq(EMBEDDING_SIZE);
})


describe("vector database should work", async () => {
    let vdb = createActor(canisterIds.vector_database_backend.local, {
        agentOptions: {
            host: "http://127.0.0.1:4943",
            fetch,
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
        let result = await vdb.query(company_id, a, 1);
        expect('Ok' in result).eq(true);
        result = result as {'Ok': Array<[number, string]>};
        const vs = result.Ok;
        expect(vs.length).eq(1);
        console.log(vs)
        console.log(values)
        expect(vs[0][1]).eq(values[0]);
    })
})
