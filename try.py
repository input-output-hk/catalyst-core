import requests
from lxml import html
import uuid

def random_string(string_length=6):
    """Returns a random string of length string_length."""
    random = str(uuid.uuid4()) # Convert UUID format to a Python string.
    random = random.replace("-","") # Remove the UUID '-'.
    return random[0:string_length] # Return the random string.

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

    def download_reviews(self, funnel_id, out_dir):
        def download_file(self, funnel_id, id, out_dir):
            export_endpoint = "/a/admin/workflow/survey-tools/assessment/report/statistic/export/assessment-details/"
            file_name = f"{funnel_id}_{random_string()}"

            res = self.client.get(f"{self.ideascale_url}{export_endpoint}{id}") 
            tree = html.fromstring(res.content)

            # we are looking for '<div class="card panel export-result-progress" data-features="refresh-processing-item" data-processing-item-id="15622">'
            # to get "15622" value as for example
            item  = int(tree.find(".//div[@data-processing-item-id]").get("data-processing-item-id"))

            export_data_endpoint = "/a/reporting/export-data/"
            while True:
                res = self.client.get(f"{ideascale_url}{export_data_endpoint}{item}")
                if "Finished Processing" in res.text:
                    download_endpoint = "/a/download-export-file/"

                    res = self.client.get(f"{ideascale_url}{download_endpoint}{item}")
                    f = open(f"{out_dir}/{file_name}.xls", "wb")
                    f.write(res.content)
                    return file_name


        funnel_endpoint = "/a/admin/workflow/stages/funnel/"

        res = self.client.get(f"{self.ideascale_url}{funnel_endpoint}{funnel_id}")
        print(res.text)
        tree = html.fromstring(res.content)

        # we are looking for '<a href="/a/admin/workflow/survey-tools/assessment/report/statistic/139?fromStage=1">Assessments</a>'
        # where we need to get url
        items = tree.findall('.//a')
        files = []
        for item in items:
            if item.text and "Assessments" in item.text:
                id = int(item.get("href").replace("/a/admin/workflow/survey-tools/assessment/report/reviews/", "").split("?")[0])

                # we are intrested in only assessed reviews 
                files.append(download_file(self, funnel_id, id, out_dir))
        return files

ideascale_url = "https://temp-cardano-sandbox.ideascale.com"
email = "alex.pozhylenkov@iohk.io"
password = "poghilenkov1995"
out_dir = "tmp"
funnel_id = 138

client = IdeaScaleClient(ideascale_url)

client.login(email, password)
files = client.download_reviews(funnel_id, out_dir)
print(len(files))
