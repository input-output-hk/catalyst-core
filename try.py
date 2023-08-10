import requests
from lxml import html

class IdeaScaleClient:

    def __init__(self, ideascale_url):
        self.ideascale_url = ideascale_url
        self.client = requests.Session()

    def login(self,  email, password):
        login = "/a/community/login"

        data = {
            'target-content-type': 'json',
            'emailAddress': 'alex.pozhylenkov@iohk.io',
            'password': 'poghilenkov1995',
            'rememberMe': 'true',
        }

        response = self.client.post(f"{self.ideascale_url}{login}", data=data)

    def get_reviews(self, id):
        export_endpoint = "/a/admin/workflow/survey-tools/assessment/report/statistic/export/assessment-details/"
    
        res = self.client.get(f"{self.ideascale_url}{export_endpoint}{id}") 
        tree = html.fromstring(res.content)

        # we are looking for '<div class="card panel export-result-progress" data-features="refresh-processing-item" data-processing-item-id="15622">'
        # to get "15622" value as for example
        item  = int(tree.find(".//div[@data-processing-item-id]").get("data-processing-item-id"))
        print(item)

        export_data_endpoint = "/a/reporting/export-data/"
        while True:
            res = self.client.get(f"{ideascale_url}{export_data_endpoint}{item}")
            if "Finished Processing" in res.text:
                download_endpoint = "/a/download-export-file/"

                url = f"{ideascale_url}{download_endpoint}{item}"

                res = self.client.get(f"{ideascale_url}{download_endpoint}{item}")
                print(len(res.content))
                f = open("try.xls", "wb")
                f.write(res.content)
                break 

ideascale_url = "https://temp-cardano-sandbox.ideascale.com"
email = "alex.pozhylenkov@iohk.io"
password = "poghilenkov1995"

client = IdeaScaleClient(ideascale_url)

client.login(email, password)
client.get_reviews(139)
