#!/usr/bin/env python3
"""
Test distributed configuration system for multi-tenant court architecture.

This test verifies:
1. Different court types get different base configurations
2. District-specific overrides work correctly
3. Configuration hierarchy is properly merged
4. Feature flags are correctly inherited
"""

import requests
import json
import sys

# Base URL for the Spin app
BASE_URL = "http://127.0.0.1:3000"

def print_test(name):
    """Print test header"""
    print(f"\n{'='*60}")
    print(f"TEST: {name}")
    print(f"{'='*60}")

def print_result(success, message):
    """Print test result"""
    status = "✅ PASS" if success else "❌ FAIL"
    print(f"{status}: {message}")
    return success

def test_district_court_config():
    """Test standard district court configuration"""
    print_test("District Court Configuration (SDNY)")

    # Test SDNY - should get district court base + SDNY overrides
    response = requests.get(
        f"{BASE_URL}/api/config/SDNY",
        headers={"X-Court-District": "SDNY"}
    )

    if response.status_code != 200:
        return print_result(False, f"Failed to get SDNY config: {response.status_code}")

    config = response.json()

    # Verify district court features are enabled
    tests_passed = True

    # Check core features
    if not config.get("features", {}).get("core", {}).get("case_management"):
        tests_passed &= print_result(False, "Core case_management should be enabled")
    else:
        tests_passed &= print_result(True, "Core case_management is enabled")

    # Check district-specific features
    if not config.get("features", {}).get("district", {}).get("criminal_cases"):
        tests_passed &= print_result(False, "District criminal_cases should be enabled")
    else:
        tests_passed &= print_result(True, "District criminal_cases is enabled")

    # Check SDNY overrides (advanced features)
    if not config.get("features", {}).get("advanced", {}).get("ai_assisted_research"):
        tests_passed &= print_result(False, "SDNY should have AI research enabled")
    else:
        tests_passed &= print_result(True, "SDNY has AI research enabled")

    # Check that bankruptcy features are NOT enabled
    if config.get("features", {}).get("bankruptcy", {}).get("chapter_7_liquidation"):
        tests_passed &= print_result(False, "District court should NOT have bankruptcy features")
    else:
        tests_passed &= print_result(True, "Bankruptcy features correctly disabled")

    return tests_passed

def test_bankruptcy_court_config():
    """Test bankruptcy court configuration"""
    print_test("Bankruptcy Court Configuration")

    # Test with bankruptcy court type header
    response = requests.get(
        f"{BASE_URL}/api/config/NYBK",
        headers={
            "X-Court-District": "NYBK",
            "X-Court-Type": "bankruptcy"
        }
    )

    if response.status_code != 200:
        return print_result(False, f"Failed to get bankruptcy config: {response.status_code}")

    config = response.json()
    tests_passed = True

    # Check bankruptcy-specific features
    if not config.get("features", {}).get("bankruptcy", {}).get("chapter_7_liquidation"):
        tests_passed &= print_result(False, "Chapter 7 should be enabled for bankruptcy court")
    else:
        tests_passed &= print_result(True, "Chapter 7 liquidation is enabled")

    if not config.get("features", {}).get("bankruptcy", {}).get("creditor_management"):
        tests_passed &= print_result(False, "Creditor management should be enabled")
    else:
        tests_passed &= print_result(True, "Creditor management is enabled")

    # Check that district court features are NOT present
    if config.get("features", {}).get("district", {}).get("criminal_cases"):
        tests_passed &= print_result(False, "Bankruptcy court should NOT have criminal cases")
    else:
        tests_passed &= print_result(True, "Criminal cases correctly disabled")

    return tests_passed

def test_fisa_court_config():
    """Test FISA court configuration with enhanced security"""
    print_test("FISA Court Configuration")

    response = requests.get(
        f"{BASE_URL}/api/config/FISA",
        headers={
            "X-Court-District": "FISA",
            "X-Court-Type": "fisa"
        }
    )

    if response.status_code != 200:
        return print_result(False, f"Failed to get FISA config: {response.status_code}")

    config = response.json()
    tests_passed = True

    # Check FISA-specific features
    if not config.get("features", {}).get("fisa", {}).get("surveillance_applications"):
        tests_passed &= print_result(False, "Surveillance applications should be enabled")
    else:
        tests_passed &= print_result(True, "Surveillance applications enabled")

    # Check enhanced security settings
    security = config.get("security", {})

    if not security.get("require_2fa"):
        tests_passed &= print_result(False, "FISA should require 2FA")
    else:
        tests_passed &= print_result(True, "2FA is required")

    if security.get("session_timeout_minutes", 60) > 15:
        tests_passed &= print_result(False, "FISA should have 15 min timeout")
    else:
        tests_passed &= print_result(True, "Session timeout is 15 minutes")

    if not security.get("require_security_clearance"):
        tests_passed &= print_result(False, "FISA should require security clearance")
    else:
        tests_passed &= print_result(True, "Security clearance required")

    return tests_passed

def test_config_hierarchy():
    """Test configuration hierarchy and overrides"""
    print_test("Configuration Hierarchy")

    # Get EDTX config (patent-heavy district)
    response = requests.get(
        f"{BASE_URL}/api/config/EDTX",
        headers={"X-Court-District": "EDTX"}
    )

    if response.status_code != 200:
        return print_result(False, f"Failed to get EDTX config: {response.status_code}")

    config = response.json()
    tests_passed = True

    # Check district info
    district_info = config.get("district_info", {})

    if district_info.get("circuit") != "5":
        tests_passed &= print_result(False, "EDTX should be in 5th Circuit")
    else:
        tests_passed &= print_result(True, "EDTX correctly in 5th Circuit")

    # Check patent-specific features
    if not config.get("features", {}).get("integrations", {}).get("uspto_integration"):
        tests_passed &= print_result(False, "EDTX should have USPTO integration")
    else:
        tests_passed &= print_result(True, "USPTO integration enabled")

    # Check local rules
    local_rules = config.get("local_rules", {})

    if not local_rules.get("patent_local_rules_enabled"):
        tests_passed &= print_result(False, "EDTX should have patent local rules")
    else:
        tests_passed &= print_result(True, "Patent local rules enabled")

    return tests_passed

def test_tax_court_config():
    """Test tax court configuration"""
    print_test("Tax Court Configuration")

    response = requests.get(
        f"{BASE_URL}/api/config/TAX",
        headers={
            "X-Court-District": "TAX",
            "X-Court-Type": "tax"
        }
    )

    if response.status_code != 200:
        return print_result(False, f"Failed to get tax court config: {response.status_code}")

    config = response.json()
    tests_passed = True

    # Check tax-specific features
    if not config.get("features", {}).get("tax", {}).get("deficiency_proceedings"):
        tests_passed &= print_result(False, "Deficiency proceedings should be enabled")
    else:
        tests_passed &= print_result(True, "Deficiency proceedings enabled")

    if not config.get("features", {}).get("tax", {}).get("s_cases"):
        tests_passed &= print_result(False, "Small tax cases should be enabled")
    else:
        tests_passed &= print_result(True, "Small tax cases enabled")

    # Check tax-specific settings
    tax_specific = config.get("tax_specific", {})

    if tax_specific.get("s_case_limit") != 50000:
        tests_passed &= print_result(False, "S case limit should be $50,000")
    else:
        tests_passed &= print_result(True, "S case limit is $50,000")

    if not tax_specific.get("s_case_no_appeal"):
        tests_passed &= print_result(False, "S cases should not be appealable")
    else:
        tests_passed &= print_result(True, "S cases cannot be appealed")

    return tests_passed

def main():
    """Run all configuration tests"""
    print("\n" + "="*60)
    print("DISTRIBUTED CONFIGURATION SYSTEM TEST SUITE")
    print("="*60)

    all_passed = True

    try:
        # Test different court types
        all_passed &= test_district_court_config()
        all_passed &= test_bankruptcy_court_config()
        all_passed &= test_fisa_court_config()
        all_passed &= test_tax_court_config()
        all_passed &= test_config_hierarchy()

    except requests.exceptions.ConnectionError:
        print("\n❌ ERROR: Could not connect to Spin app at", BASE_URL)
        print("Make sure the app is running with: spin up")
        return 1
    except Exception as e:
        print(f"\n❌ ERROR: {e}")
        return 1

    # Print summary
    print("\n" + "="*60)
    if all_passed:
        print("✅ ALL TESTS PASSED!")
        print("The distributed configuration system is working correctly.")
    else:
        print("❌ SOME TESTS FAILED")
        print("Please review the failures above.")
    print("="*60)

    return 0 if all_passed else 1

if __name__ == "__main__":
    sys.exit(main())