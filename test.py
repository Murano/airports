import json
import asyncio
import aiohttp


async def req():
    resp = await aiohttp.ClientSession().request(
        "post", 'http://127.0.0.1:8080/batch_insert',
        data=json.dumps({"tickets":[{
            "id": "51e91cabbc513365f132b449742220d3",
            "departure_code": "LED",
            "arrival_code": "DME",
            "departure_time": 1509876000,
            "arrival_time": 1509883200,
            "price": 1500
        },

            {
                "id": "900b49120b93d07b2f69316a843abba1",
                "departure_code": "DME",
                "arrival_code": "AER",
                "departure_time": 1509904800,
                "arrival_time": 1509915600,
                "price": 2000
            }]}),
        headers={"content-type": "application/json"})
    print(str(resp))
    print(await resp.text())
    # assert 200 == resp.status


async def search():
    resp = await aiohttp.ClientSession().request(
        "post", 'http://127.0.0.1:8080/search',
        data=json.dumps({
            "departure_code": "LED",
            "arrival_code": "AER",
            "departure_time_start": 1509840000,
            "departure_time_end": 1509926399
        }),
        headers={"content-type": "application/json"})
    # print(str(resp))
    print(await resp.text())
    assert 200 == resp.status


asyncio.get_event_loop().run_until_complete(req())
asyncio.get_event_loop().run_until_complete(search())
