import http from 'k6/http';
import { sleep, check } from 'k6';

export const options = {
  vus: 100,           // Virtual users
  duration: '30s',    // Test duration
  thresholds: {
    http_req_duration: ['p(95)<500'], // 95% of requests must complete below 500ms
  },
};

// Test user creation and login
export default function () {
  // Generate unique username
  const username = `user_${Date.now()}_${Math.floor(Math.random() * 10000)}`;
  
  // Register a new user
  const registerRes = http.post('http://localhost:8080/api/v1/users/register', JSON.stringify({
    username: username,
    email: `${username}@example.com`,
    password: 'securepassword123',
    first_name: 'Load',
    last_name: 'Test'
  }), {
    headers: { 'Content-Type': 'application/json' },
  });
  
  check(registerRes, {
    'register success': (r) => r.status === 200,
    'register time OK': (r) => r.timings.duration < 300
  });
  
  sleep(1);
  
  // Login with created user
  const loginRes = http.post('http://localhost:8080/api/v1/users/login', JSON.stringify({
    username: username,
    password: 'securepassword123',
  }), {
    headers: { 'Content-Type': 'application/json' },
  });
  
  check(loginRes, {
    'login success': (r) => r.status === 200,
    'login time OK': (r) => r.timings.duration < 200
  });
  
  // Extract token for authenticated requests
  if (loginRes.status === 200) {
    try {
      const body = JSON.parse(loginRes.body);
      const token = body.data.token;
      
      // Get user accounts
      const accountsRes = http.get('http://localhost:8080/api/v1/accounts', {
        headers: {
          'Authorization': `Bearer ${token}`,
          'Content-Type': 'application/json'
        },
      });
      
      check(accountsRes, {
        'get accounts success': (r) => r.status === 200,
        'accounts retrieved': (r) => {
          const body = JSON.parse(r.body);
          return body.data && Array.isArray(body.data);
        }
      });
      
      // Create deposit - simulating financial transaction
      if (accountsRes.status === 200) {
        try {
          const accountsBody = JSON.parse(accountsRes.body);
          if (accountsBody.data && accountsBody.data.length > 0) {
            const accountId = accountsBody.data[0].id;
            
            const depositRes = http.post('http://localhost:8080/api/v1/transactions/deposit', 
              JSON.stringify({
                account_id: accountId,
                amount: "100.00",
                description: "Load test deposit"
              }), {
                headers: {
                  'Authorization': `Bearer ${token}`,
                  'Content-Type': 'application/json'
                },
              }
            );
            
            check(depositRes, {
              'deposit success': (r) => r.status === 200,
              'deposit time OK': (r) => r.timings.duration < 300
            });
          }
        } catch (e) {
          console.log('Error processing accounts', e);
        }
      }
      
    } catch (e) {
      console.log('Error parsing login response', e);
    }
  }
  
  sleep(1);
} 