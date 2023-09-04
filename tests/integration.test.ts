import { expect, test, describe, it} from "vitest";
import { Actor, CanisterStatus, HttpAgent } from "@dfinity/agent";
import { Principal } from "@dfinity/principal";
import {array, random} from 'vectorious';
import {createActor} from '../src/declarations/vector_database_backend'
import canisterIds from '../.dfx/local/canister_ids.json';

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
            fetch: fetch
        }
    });

    let a = genRandomEmbedding();
    let b = genRandomEmbedding();
    let c = genRandomEmbedding();
    let embds = [a, b, c];
    let values = [
        "question 1?",
        "question 4?",
        "question 6?",
    ];
    it("should accept embeddings", async () => {
        const result = await vdb.append_keys_values(embds, values);
        expect('Ok' in result).eq(true);
    })

    it("should build index", async () => {
        const result = await vdb.build_index();
        expect('Ok' in result).eq(true);
    })

    it("should return a same value corresponding to embedding", async () => {
        // embedding, limit
        let result = await vdb.query(a, 1);
        expect('Ok' in result).eq(true);
        result = result as {'Ok': Array<string>};
        const values = result.Ok;
        expect(values.length).eq(1);
        expect(values[0]).eq(values[0]);
    })
})
