#!/bin/bash

echo "Running Transaction Manager Performance Tests"
echo "--------------------------------------------"

# Check if hey is installed
if ! command -v hey &> /dev/null; then
    echo "The 'hey' HTTP load testing tool is not installed."
    echo "Install it with: go install github.com/rakyll/hey@latest"
    
    # Check if ab is available as an alternative
    if command -v ab &> /dev/null; then
        echo "Using Apache Bench (ab) as an alternative."
        USE_AB=true
    else
        echo "Apache Bench (ab) is also not available."
        echo "Please install either 'hey' or Apache Bench to run performance tests."
        exit 1
    fi
fi

# Check if the server is running
echo "Checking if server is running..."
if ! curl -s http://localhost:8080/ > /dev/null; then
    echo "Error: Server is not running on http://localhost:8080/"
    echo "Please start the server with: cargo run --release"
    exit 1
fi
echo "Server is running. Starting performance tests..."

# Function to run tests with hey or ab
run_test() {
    local name=$1
    local method=$2
    local url=$3
    local data=$4
    local concurrency=${5:-50}
    local requests=${6:-200}
    
    echo ""
    echo "Running test: $name"
    echo "URL: $url"
    echo "Method: $method"
    echo "Concurrency: $concurrency"
    echo "Requests: $requests"
    
    if [ "$USE_AB" = true ]; then
        # Use Apache Bench
        if [ "$method" = "GET" ]; then
            ab -n $requests -c $concurrency -q "$url"
        else
            # For POST requests, create a temp file with the data
            echo "$data" > /tmp/ab_data.json
            ab -n $requests -c $concurrency -q -p /tmp/ab_data.json \
               -T "application/json" "$url"
            rm /tmp/ab_data.json
        fi
    else
        # Use hey
        if [ "$method" = "GET" ]; then
            hey -n $requests -c $concurrency "$url"
        else
            hey -n $requests -c $concurrency -m $method \
                -H "Content-Type: application/json" -d "$data" "$url"
        fi
    fi
    
    echo "Completed test: $name"
    echo "--------------------------------------------------"
}

# Test the health endpoint
run_test "Health Endpoint" "GET" "http://localhost:8080/" "" 50 200

# Test user registration
USER_JSON='{"username":"perftest_'$(date +%s)'","email":"perf_'$(date +%s)'@example.com","password":"securepassword","first_name":"Performance","last_name":"Test"}'
run_test "User Registration" "POST" "http://localhost:8080/api/v1/users/register" "$USER_JSON" 10 50

# Test login - after creating a test user
echo "Creating test user for login performance testing..."
TEST_USER='{
  "username":"loadtestuser_'$(date +%s)'",
  "email":"loadtestuser_'$(date +%s)'@example.com",
  "password":"securepassword",
  "first_name":"Load",
  "last_name":"Test"
}'

curl -s -X POST -H "Content-Type: application/json" \
     -d "$TEST_USER" \
     http://localhost:8080/api/v1/users/register > /dev/null

# Extract username and password from the TEST_USER variable
USERNAME=$(echo $TEST_USER | grep -o '"username":"[^"]*' | cut -d'"' -f4)
PASSWORD=$(echo $TEST_USER | grep -o '"password":"[^"]*' | cut -d'"' -f4)

# Test login performance
LOGIN_JSON='{"username":"'$USERNAME'","password":"'$PASSWORD'"}'
run_test "User Login" "POST" "http://localhost:8080/api/v1/users/login" "$LOGIN_JSON" 20 100

echo ""
echo "Performance testing complete!"
echo "For more comprehensive load testing, use k6 with the load-test.js script:"
echo "  k6 run load-test.js" 