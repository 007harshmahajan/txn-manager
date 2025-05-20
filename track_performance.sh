#!/bin/bash

# Script to track performance improvements over time

TIMESTAMP=$(date +%Y%m%d_%H%M%S)
OUTPUT_DIR="performance_results"
OUTPUT_FILE="${OUTPUT_DIR}/perf_test_${TIMESTAMP}.txt"

# Create output directory if it doesn't exist
mkdir -p "${OUTPUT_DIR}"

echo "Transaction Manager Performance Tracking"
echo "========================================"
echo "Running performance tests and saving results to: ${OUTPUT_FILE}"

# Start the output file
cat > "${OUTPUT_FILE}" << EOF
# Transaction Manager Performance Test
Date: $(date)
Git Commit: $(git rev-parse HEAD)
Git Branch: $(git branch --show-current)

## System Information
OS: $(uname -a)
CPU: $(grep "model name" /proc/cpuinfo | head -n 1 | cut -d ":" -f 2 | xargs)
Memory: $(grep MemTotal /proc/meminfo | awk '{print $2 / 1024 / 1024 " GB"}')

## Test Parameters
- K6 Tests: 100 VUs, 30s duration
- HTTP Endpoints: health, registration, login, accounts, transactions

## Results
EOF

# Check if the server is running
if ! curl -s http://localhost:8080/ > /dev/null; then
    echo "ERROR: Server is not running on http://localhost:8080/"
    echo "Please start the server with: cargo run --release"
    exit 1
fi

# Run the performance tests
echo "Running basic performance tests..."
./run_performance_tests.sh | tee -a "${OUTPUT_FILE}"

# Run k6 tests if available
if command -v k6 &> /dev/null; then
    echo "Running k6 load tests..."
    echo -e "\n## K6 Load Test Results\n" >> "${OUTPUT_FILE}"
    k6 run load-test.js | tee -a "${OUTPUT_FILE}"
else
    echo "K6 not found. Skipping load tests."
    echo "To install k6, follow the instructions in BUILDING.md"
fi

# Compare with previous results if available
PREV_RESULT=$(ls -t "${OUTPUT_DIR}"/perf_test_*.txt 2>/dev/null | sed -n '2p')
if [ -n "${PREV_RESULT}" ] && [ -f "${PREV_RESULT}" ]; then
    echo -e "\n## Comparison with Previous Test ($(basename ${PREV_RESULT}))\n" >> "${OUTPUT_FILE}"
    
    # Extract key metrics for comparison
    echo "Comparing with previous test from: $(grep "Date:" "${PREV_RESULT}" | cut -d ":" -f 2- | xargs)"
    
    # Compare health endpoint latency
    CURRENT_HEALTH_LATENCY=$(grep -A 20 "Health Endpoint" "${OUTPUT_FILE}" | grep "Average:" | head -n 1 | awk '{print $2}')
    PREVIOUS_HEALTH_LATENCY=$(grep -A 20 "Health Endpoint" "${PREV_RESULT}" | grep "Average:" | head -n 1 | awk '{print $2}')
    
    # Compare login latency
    CURRENT_LOGIN_LATENCY=$(grep -A 20 "User Login" "${OUTPUT_FILE}" | grep "Average:" | head -n 1 | awk '{print $2}')
    PREVIOUS_LOGIN_LATENCY=$(grep -A 20 "User Login" "${PREV_RESULT}" | grep "Average:" | head -n 1 | awk '{print $2}')
    
    # Add comparison to output file
    cat >> "${OUTPUT_FILE}" << EOF
| Metric | Previous | Current | Change |
|--------|----------|---------|--------|
| Health Endpoint Latency | ${PREVIOUS_HEALTH_LATENCY:-N/A} | ${CURRENT_HEALTH_LATENCY:-N/A} | $(if [[ -n "$PREVIOUS_HEALTH_LATENCY" && -n "$CURRENT_HEALTH_LATENCY" ]]; then python3 -c "print('%.2f%%' % ((float('${PREVIOUS_HEALTH_LATENCY}') - float('${CURRENT_HEALTH_LATENCY}')) / float('${PREVIOUS_HEALTH_LATENCY}') * 100))"; else echo "N/A"; fi) |
| Login Endpoint Latency | ${PREVIOUS_LOGIN_LATENCY:-N/A} | ${CURRENT_LOGIN_LATENCY:-N/A} | $(if [[ -n "$PREVIOUS_LOGIN_LATENCY" && -n "$CURRENT_LOGIN_LATENCY" ]]; then python3 -c "print('%.2f%%' % ((float('${PREVIOUS_LOGIN_LATENCY}') - float('${CURRENT_LOGIN_LATENCY}')) / float('${PREVIOUS_LOGIN_LATENCY}') * 100))"; else echo "N/A"; fi) |
EOF

    # If k6 results are available, compare them too
    if grep -q "http_req_duration" "${OUTPUT_FILE}" && grep -q "http_req_duration" "${PREV_RESULT}"; then
        CURRENT_P95=$(grep -A 1 "http_req_duration" "${OUTPUT_FILE}" | grep "p(95)" | sed -E 's/.*p\(95\)=([0-9.]+)s.*/\1/')
        PREVIOUS_P95=$(grep -A 1 "http_req_duration" "${PREV_RESULT}" | grep "p(95)" | sed -E 's/.*p\(95\)=([0-9.]+)s.*/\1/')
        
        CURRENT_AVG=$(grep "http_req_duration" "${OUTPUT_FILE}" | grep "avg=" | sed -E 's/.*avg=([0-9.]+)s.*/\1/')
        PREVIOUS_AVG=$(grep "http_req_duration" "${PREV_RESULT}" | grep "avg=" | sed -E 's/.*avg=([0-9.]+)s.*/\1/')
        
        cat >> "${OUTPUT_FILE}" << EOF
| P95 Response Time | ${PREVIOUS_P95:-N/A}s | ${CURRENT_P95:-N/A}s | $(if [[ -n "$PREVIOUS_P95" && -n "$CURRENT_P95" ]]; then python3 -c "print('%.2f%%' % ((float('${PREVIOUS_P95}') - float('${CURRENT_P95}')) / float('${PREVIOUS_P95}') * 100))"; else echo "N/A"; fi) |
| Avg Response Time | ${PREVIOUS_AVG:-N/A}s | ${CURRENT_AVG:-N/A}s | $(if [[ -n "$PREVIOUS_AVG" && -n "$CURRENT_AVG" ]]; then python3 -c "print('%.2f%%' % ((float('${PREVIOUS_AVG}') - float('${CURRENT_AVG}')) / float('${PREVIOUS_AVG}') * 100))"; else echo "N/A"; fi) |
EOF
    fi
    
    echo "Comparison with previous test completed."
fi

echo -e "\n## Recommendations\n" >> "${OUTPUT_FILE}"
echo "Based on these results, consider the following optimizations:" >> "${OUTPUT_FILE}"

# Add basic recommendations based on results
if grep -q "p(95)>[0-9]" "${OUTPUT_FILE}"; then
    echo "1. Further optimize database connection pooling (see PERFORMANCE.md)" >> "${OUTPUT_FILE}"
fi

if grep -q "500.*responses" "${OUTPUT_FILE}"; then
    echo "2. Improve error handling and add retry logic for database operations" >> "${OUTPUT_FILE}"
fi

if grep -q "login.*time.*[0-5][0-9][0-9]ms" "${OUTPUT_FILE}"; then
    echo "3. Consider implementing authentication caching" >> "${OUTPUT_FILE}"
fi

echo "Performance test complete! Results saved to: ${OUTPUT_FILE}"
echo "For detailed analysis and improvement recommendations, see PERFORMANCE.md" 