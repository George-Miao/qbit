# VERSION: 1.0
# AUTHORS: qbit-rs

from novaprinter import prettyPrinter


class qbit_rs_test(object):
    url = "http://example.invalid"
    name = "qbit-rs test"
    supported_categories = {"all": "all"}

    def search(self, what, cat="all"):
        prettyPrinter({
            "url": "magnet:?xt=urn:btih:722fe65b2aa26d14f35b4ad627d20236e481d924",
            "name": "qbit-rs deterministic result",
            "size": 163783,
            "seeds": 1,
            "leech": 0,
            "engine_url": self.url,
            "desc_link": "http://example.invalid/result",
        })
