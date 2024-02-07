import csv
from pymongo import MongoClient


def main():

    # Create an empty list to store the leak_ids
    leak_ids = []

    # Open the CSV file for reading
    with open('large.csv', mode='r', newline='') as file:
        # Create a CSV reader object
        csv_reader = csv.DictReader(file)

        # Iterate through the rows in the CSV file
        for row in csv_reader:
            # Extract the "leak_id" from each row and add it to the list
            leak_id = row['leak_id']
            leak_ids.append(leak_id)

    # MongoDB connection settings
    mongo_host = 'localhost'  # Change this to your MongoDB server address
    mongo_port = 27017  # Change this to your MongoDB server port

    # Connect to MongoDB
    client = MongoClient(mongo_host, mongo_port)

    # Database and collection
    db = client['leaks']
    collection = db['identities']

    # MongoDB query
    query = {"leak_id": {"$in": leak_ids}, "password": { "$exists": True, "$ne": [] }, "$or": [{ "email": { "$exists": True, "$ne": [] } }, { "phone": { "$exists": True, "$ne": [] } },]}

    # Retrieve documents from MongoDB
    documents = collection.find(query)
    document_count = collection.count_documents(query)

    print(f"Actually, only {document_count} identities match our filter")

    # Close the MongoDB connection
    client.close()

if __name__ == "__main__":
    main()