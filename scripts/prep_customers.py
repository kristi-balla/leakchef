import csv
from pymongo import MongoClient
import uuid


def main():

    # MongoDB connection settings
    mongo_host = 'localhost'  # Change this to your MongoDB server address
    mongo_port = 27017  # Change this to your MongoDB server port

    # Connect to MongoDB
    client = MongoClient(mongo_host, mongo_port)

    # Database and collection
    db = client['leaks']
    collection = db['customers']

    # MongoDB query
    query = {}
    update = {"$set": {"handled_leaks": []}}

    # Retrieve documents from MongoDB
    documents = collection.update_many(query, update)

    for i in range(16):
        customer = {"api_key": str(uuid.uuid4()), "customer_id": i, "handled_leaks": [], "customer_salt": "BjjLnn7"}
        collection.insert_one(customer)
    
    # Close the MongoDB connection
    client.close()

if __name__ == "__main__":
    main()