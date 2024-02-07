import csv
from pymongo import MongoClient


def main():

    # MongoDB connection settings
    mongo_host = 'localhost'  # Change this to your MongoDB server address
    mongo_port = 27017  # Change this to your MongoDB server port

    # Connect to MongoDB
    client = MongoClient(mongo_host, mongo_port)

    # Database and collection
    db = client['leaks']
    collection = db['metadata']

    # MongoDB query
    query = {"parsed_identities": {"$gt": 100000}}

    # Retrieve documents from MongoDB
    documents = collection.find(query)
    document_count = collection.count_documents(query)

    # Define the CSV filename
    csv_filename = "large.csv"

    sum = 0

    # Write the documents to a CSV file
    with open(csv_filename, 'w', newline='') as csv_file:
        fieldnames = list(documents[0].keys()) if document_count > 0 else []  # Get field names from the first document
        writer = csv.DictWriter(csv_file, fieldnames=fieldnames)
        writer.writeheader()
        for doc in documents:
            sum = sum + doc.get("parsed_identities")
            writer.writerow(doc)

    print(f"Exported {document_count} documents to {csv_filename}")
    print(f"Currently working with {sum} identities")

    # Close the MongoDB connection
    client.close()

if __name__ == "__main__":
    main()