#include <cmath>
#include <iomanip>
#include <iostream>
#include <random>
#include <string>

struct Vec3 { double x, y, z; };

static void emit_observation(const std::string& sensor, const std::string& target,
                             long long timestamp_ms, Vec3 p, Vec3 v, double confidence) {
    std::cout << std::fixed << std::setprecision(3)
              << "{\"kind\":\"observation\",\"schema_version\":1,\"event_id\":\"obs-"
              << timestamp_ms << "-" << sensor << "\",\"sensor\":\"" << sensor
              << "\",\"target_id\":\"" << target
              << "\",\"timestamp_ms\":" << timestamp_ms
              << ",\"position\":{\"x_m\":" << p.x << ",\"y_m\":" << p.y << ",\"z_m\":" << p.z << "}"
              << ",\"velocity\":{\"x_mps\":" << v.x << ",\"y_mps\":" << v.y << ",\"z_mps\":" << v.z << "}"
              << ",\"confidence\":" << confidence << "}\n";
}

int main() {
    std::mt19937 rng(7); // deterministic test scenario
    std::normal_distribution<double> noise(0.0, 2.0);

    constexpr int steps = 12;
    for (int step = 0; step < steps; ++step) {
        const long long t = step * 1000;
        const double progress = static_cast<double>(step);
        const Vec3 truth{120.0 - progress * 11.0, 35.0 + progress * 1.2, 80.0};
        const Vec3 velocity{-11.0, 1.2, 0.0};

        emit_observation("radar", "sim-01", t,
                         {truth.x + noise(rng), truth.y + noise(rng), truth.z + noise(rng)}, velocity, 0.86);
        emit_observation("rf", "sim-01", t,
                         {truth.x + noise(rng) * 1.5, truth.y + noise(rng) * 1.5, truth.z}, velocity, 0.72);

        if (step % 2 == 0) {
            emit_observation("eo", "sim-01", t,
                             {truth.x + noise(rng) * 0.7, truth.y + noise(rng) * 0.7, truth.z + noise(rng) * 0.7}, velocity, 0.91);
        }
    }
    return 0;
}
