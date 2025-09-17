#!/bin/bash

# Test script for ToDo API pagination
# Make sure the API is running first with: spin up

API_URL="http://localhost:3000"

echo "🧪 Testing ToDo API Pagination"
echo "================================"

# Step 1: Create multiple ToDo items
echo -e "\n📝 Creating 25 test ToDo items..."

for i in {1..25}; do
    response=$(curl -s -X POST "$API_URL/api/todos" \
        -H "Content-Type: application/json" \
        -d "{\"contents\": \"Test ToDo Item #$i\"}")

    if [ $? -eq 0 ]; then
        echo -n "."
    else
        echo -n "X"
    fi
done

echo -e "\n✅ Test data created"

# Step 2: Test pagination - First page
echo -e "\n📄 Testing Page 1 (default pagination)..."
echo "GET $API_URL/api/todos"
curl -s "$API_URL/api/todos" | jq '.'

# Step 3: Test pagination - Page 2 with limit 5
echo -e "\n📄 Testing Page 2 with limit=5..."
echo "GET $API_URL/api/todos?page=2&limit=5"
curl -s "$API_URL/api/todos?page=2&limit=5" | jq '.'

# Step 4: Test filtering - Only incomplete todos
echo -e "\n🔍 Testing filter: completed=false with limit=10..."
echo "GET $API_URL/api/todos?completed=false&limit=10"
curl -s "$API_URL/api/todos?completed=false&limit=10" | jq '.'

# Step 5: Mark some as completed and test filtering
echo -e "\n✅ Marking some ToDos as completed..."
# Get all todos first to get their IDs
todos=$(curl -s "$API_URL/api/todos?limit=5" | jq -r '.items[].id')

count=0
for id in $todos; do
    if [ $count -lt 3 ]; then
        curl -s -X POST "$API_URL/api/todos/$id/toggle" > /dev/null
        echo "Toggled ToDo: $id"
        count=$((count + 1))
    fi
done

# Step 6: Test filtering - Only completed todos
echo -e "\n🔍 Testing filter: completed=true..."
echo "GET $API_URL/api/todos?completed=true"
curl -s "$API_URL/api/todos?completed=true" | jq '.'

# Step 7: Test pagination limits
echo -e "\n🔢 Testing different page sizes..."
echo "GET $API_URL/api/todos?limit=3"
curl -s "$API_URL/api/todos?limit=3" | jq '{total, totalPages, hasNext, hasPrevious, itemCount: (.items | length)}'

echo -e "\nGET $API_URL/api/todos?page=3&limit=3"
curl -s "$API_URL/api/todos?page=3&limit=3" | jq '{page, total, totalPages, hasNext, hasPrevious, itemCount: (.items | length)}'

# Step 8: Test error handling
echo -e "\n❌ Testing error cases..."
echo "Invalid page number (page=0):"
curl -s "$API_URL/api/todos?page=0" | jq '.'

echo -e "\nInvalid limit (limit=200):"
curl -s "$API_URL/api/todos?limit=200" | jq '.'

echo -e "\nInvalid completed filter:"
curl -s "$API_URL/api/todos?completed=maybe" | jq '.'

# Step 9: Health check
echo -e "\n❤️  Testing health endpoint..."
echo "GET $API_URL/api/health"
curl -s "$API_URL/api/health" | jq '.'

echo -e "\n✨ Pagination tests completed!"