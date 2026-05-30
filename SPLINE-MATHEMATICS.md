# Spline-Based Continuous Embeddings and Fluid Vector Spaces for Room-Level Understanding

**PLATO Nervous System — Mathematical Foundations Report**

*May 2026*

---

## Abstract

This report develops the mathematical framework for representing a PLATO room's evolving understanding as a continuously parameterized trajectory through a high-dimensional embedding manifold. We formalize sensor-driven updates as spline refinements, develop online manifold-learning procedures for streaming data, and introduce a Hamiltonian mechanics of inter-room correlation. The result is a rigorous foundation for time-indexed, cross-room queries via a Joint-Embedding Predictive Architecture (JEPA).

---

## 1. Spline Mathematics for Embedding Trajectories

### 1.1 Problem Setting

Let $\mathcal{R}$ denote a PLATO room. At each sensor tick $t_k$ ($k = 0, 1, 2, \ldots$), the room produces an embedding vector $\mathbf{v}_k \in \mathbb{R}^d$, where $d$ is the dimensionality of the latent space. We seek a continuous map

$$\gamma : [t_0, t_N] \longrightarrow \mathbb{R}^d$$

such that $\gamma(t_k) = \mathbf{v}_k$ for all observed ticks, and $\gamma$ is smooth in an appropriate sense.

### 1.2 B-Spline Construction

A B-spline of degree $p$ on the knot vector $\mathbf{T} = [\tau_0, \tau_1, \ldots, \tau_{m}]$ is defined by

$$\gamma(t) = \sum_{i=0}^{n-1} \mathbf{c}_i \, B_{i,p}(t)$$

where $\mathbf{c}_i \in \mathbb{R}^d$ are control points and $B_{i,p}$ are the Cox–de Boor basis functions:

$$B_{i,0}(t) = \begin{cases} 1 & \text{if } \tau_i \leq t < \tau_{i+1} \\ 0 & \text{otherwise} \end{cases}$$

$$B_{i,p}(t) = \frac{t - \tau_i}{\tau_{i+p} - \tau_i} B_{i,p-1}(t) + \frac{\tau_{i+p+1} - t}{\tau_{i+p+1} - \tau_{i+1}} B_{i+1,p-1}(t)$$

For embedding trajectories, we choose $p = 3$ (cubic B-splines), which guarantees $C^2$ continuity — sufficient for well-defined curvature analysis.

### 1.3 NURBS Generalization

Non-Uniform Rational B-Splines (NURBS) extend B-splines with weights $w_i > 0$:

$$\gamma(t) = \frac{\sum_{i=0}^{n-1} w_i \, \mathbf{c}_i \, B_{i,p}(t)}{\sum_{i=0}^{n-1} w_i \, B_{i,p}(t)}$$

This allows exact representation of conic sections in the embedding space, which is useful when room understanding follows curved trajectories (e.g., cyclical daily patterns that approximate ellipses). Setting $w_i = 1$ for all $i$ recovers the standard B-spline.

### 1.4 Fitting the Spline to Observations

Given observations $\{(t_k, \mathbf{v}_k)\}_{k=0}^{N}$, we solve for control points via least squares with a regularization term:

$$\min_{\mathbf{c}_0, \ldots, \mathbf{c}_{n-1}} \sum_{k=0}^{N} \left\| \mathbf{v}_k - \sum_{i} \mathbf{c}_i B_{i,p}(t_k) \right\|^2 + \lambda \int \left\| \gamma''(t) \right\|^2 dt$$

The parameter $\lambda$ controls smoothness: $\lambda = 0$ interpolates exactly; $\lambda \to \infty$ forces a straight line. The integral penalizes total curvature, favoring smoother trajectories. This is the **smoothing spline** formulation extended to $\mathbb{R}^d$.

### 1.5 Derivatives: Rate of Change of Understanding

The first derivative of the spline,

$$\gamma'(t) = \sum_{i} \mathbf{c}_i \, B'_{i,p}(t)$$

represents the **instantaneous rate of change** of the room's understanding. When $\|\gamma'(t)\|$ is large, the room's embedding is shifting rapidly — a period of rapid learning or environmental change. When $\|\gamma'(t)\| \approx 0$, the room is in a stable state.

### 1.6 Curvature: How Rapidly the "Vibe" Shifts

The curvature of the embedding trajectory is

$$\kappa(t) = \frac{\|\gamma'(t) \times \gamma''(t)\|}{\|\gamma'(t)\|^3}$$

In higher dimensions ($d > 3$), the cross product generalizes via the Gram determinant:

$$\kappa(t) = \frac{\sqrt{\|\gamma'\|^2 \|\gamma''\|^2 - \langle \gamma', \gamma'' \rangle^2}}{\|\gamma'(t)\|^3}$$

High curvature $\kappa$ indicates the room's trajectory is **turning sharply** — its character is undergoing a qualitative shift. Low curvature indicates gradual, predictable evolution.

### 1.7 Inflection Points: Phase Transitions

An inflection point occurs where the curvature changes sign, i.e., where $\gamma''(t) \cdot \gamma'(t)^\perp$ crosses zero. Formally, we seek $t^*$ such that:

$$\frac{d}{dt}\left[\frac{\|\gamma'\|^2 \|\gamma''\|^2 - \langle \gamma', \gamma'' \rangle^2}{\|\gamma'\|^6}\right]_{t=t^*} = 0 \quad \text{and curvature changes sign}$$

These inflection points are **phase transitions** in the room's understanding — moments where the dominant pattern shifts from accelerating change to decelerating change or vice versa. Detecting these in real-time is valuable: they mark when a room's character fundamentally transforms.

---

## 2. Manifold Learning on Streaming Data

### 2.1 The Room's Understanding Surface

We hypothesize that the embedding vectors $\{\mathbf{v}_k\}$ lie on or near a low-dimensional manifold $\mathcal{M} \subset \mathbb{R}^d$ with $\dim(\mathcal{M}) = m \ll d$. This manifold is the room's **understanding surface** — the intrinsic geometry that captures the room's true degrees of freedom.

### 2.2 Online UMAP

Standard UMAP operates in batch mode. For streaming data, we adapt the algorithm as follows:

**Initialization.** From the first $N_0$ observations $\{\mathbf{v}_0, \ldots, \mathbf{v}_{N_0-1}\}$, compute the standard UMAP embedding $\{y_0, \ldots, y_{N_0-1}\} \subset \mathbb{R}^{m}$ with fuzzy simplicial set $\mathcal{F}$.

**Online Update.** When a new vector $\mathbf{v}_{new}$ arrives:

1. **Find $k$-nearest neighbors** $\{\mathbf{v}_{i_1}, \ldots, \mathbf{v}_{i_k}\}$ in the existing set.
2. **Compute membership strengths** $\mu_j = \exp\left(-\frac{d(\mathbf{v}_{new}, \mathbf{v}_{i_j}) - \rho_{new}}{\sigma_{new}}\right)$ where $\rho_{new} = \min_j d(\mathbf{v}_{new}, \mathbf{v}_{i_j})$ and $\sigma_{new}$ is chosen so $\sum_j \mu_j = \log_2 k$.
3. **Initialize embedding** $y_{new} = \frac{1}{k} \sum_{j=1}^{k} y_{i_j}$ (barycentric average).
4. **Optimize** $y_{new}$ by minimizing cross-entropy against the existing fuzzy simplicial set:

$$\min_{y_{new}} \sum_{j=1}^{k} \left[ -\mu_j \log\left(\sigma(y_{new}, y_{i_j})\right) - (1-\mu_j)\log\left(1 - \sigma(y_{new}, y_{i_j})\right) \right]$$

where $\sigma(a,b) = \left(1 + \|a-b\|^{2}\right)^{-1}$.

5. **Optionally refine** nearby embeddings by performing $S$ steps of stochastic gradient descent on the cross-entropy with respect to $\{y_{i_1}, \ldots, y_{i_k}\}$ as well.

This produces an **incrementally growing manifold** that tracks the room's evolving understanding.

### 2.3 Geodesic Distance

The geodesic distance $d_{\mathcal{M}}(\mathbf{v}_a, \mathbf{v}_b)$ between two room states on the manifold approximates the true distance more faithfully than Euclidean distance in the ambient space. On the fuzzy simplicial complex, we compute:

$$d_{\mathcal{M}}(\mathbf{v}_a, \mathbf{v}_b) = \min_{\text{paths}} \sum_{(i,j) \in \text{path}} -\log\left(\mu_{ij}\right)$$

This is the shortest-path distance in the weighted graph induced by the fuzzy simplicial set.

### 2.4 Tangent Spaces

At any point $\mathbf{v} \in \mathcal{M}$, the tangent space $T_{\mathbf{v}}\mathcal{M}$ provides a local linear approximation:

$$T_{\mathbf{v}}\mathcal{M} = \text{span}\{\mathbf{e}_1(\mathbf{v}), \ldots, \mathbf{e}_m(\mathbf{v})\}$$

where $\{\mathbf{e}_j(\mathbf{v})\}$ are the $m$ leading eigenvectors of the local covariance matrix:

$$\Sigma_{\mathbf{v}} = \frac{1}{k} \sum_{j=1}^{k} (\mathbf{v}_{i_j} - \mathbf{v})(\mathbf{v}_{i_j} - \mathbf{v})^\top$$

The tangent space encodes which directions of variation are meaningful at the current room state. Movement within $T_{\mathbf{v}}\mathcal{M}$ corresponds to natural room evolution; movement orthogonal to $T_{\mathbf{v}}\mathcal{M}$ indicates anomalous or novel states.

---

## 3. Dynamic Correlation and "Magnetism"

### 3.1 Inter-Room Correlation as a Force

Consider two rooms $\mathcal{R}_A$ and $\mathcal{R}_B$ with embedding trajectories $\gamma_A(t)$ and $\gamma_B(t)$. We define their **instantaneous correlation** as:

$$\rho_{AB}(t) = \frac{\langle \gamma_A(t) - \bar{\gamma}_A, \gamma_B(t) - \bar{\gamma}_B \rangle}{\|\gamma_A(t) - \bar{\gamma}_A\| \cdot \|\gamma_B(t) - \bar{\gamma}_B\|}$$

where $\bar{\gamma}_A$ and $\bar{\gamma}_B$ are running means over a suitable window.

### 3.2 Correlation Force

The **correlation force** between rooms is the time derivative:

$$\mathbf{F}_{AB}(t) = \nabla_{\gamma_A} \dot{\rho}_{AB}(t) = \frac{d}{dt} \rho_{AB}(t) \cdot \hat{n}_{AB}(t)$$

where $\hat{n}_{AB}$ is the unit vector from $\gamma_A$ toward $\gamma_B$ in the embedding space. When $\dot{\rho}_{AB} > 0$, rooms are **becoming more similar** (attractive force); when $\dot{\rho}_{AB} < 0$, they are **diverging** (repulsive force).

### 3.3 Coulomb-like Repulsion

When rooms diverge beyond a threshold, we model repulsion analogously to Coulomb's law:

$$\mathbf{F}_{AB}^{\text{repulsive}} = -k_e \frac{q_A \, q_B}{\|\gamma_A - \gamma_B\|^2} \hat{r}_{AB}$$

where $q_A, q_B$ are "charges" proportional to each room's informational entropy $H(\mathcal{R}_i)$, and $k_e$ is a coupling constant. High-entropy rooms repel more strongly — they represent fundamentally different types of understanding.

### 3.4 Hamiltonian Formulation

The total energy of the room system is:

$$\mathcal{H} = \sum_{i} \frac{1}{2} m_i \|\dot{\gamma}_i\|^2 + \sum_{i < j} V_{ij}(\|\gamma_i - \gamma_j\|)$$

where $m_i$ is the "mass" of room $i$ (proportional to its data density or confidence), and $V_{ij}$ is the interaction potential:

$$V_{ij}(r) = -\alpha_{ij} \log(r) + \frac{\beta_{ij}}{r} + \gamma_{ij} \, r^2$$

This combines:
- **Logarithmic attraction** (rooms with shared structure tend to cluster)
- **Inverse-distance repulsion** (prevents collapse to a point)
- **Quadratic confinement** (rooms stay within a bounded region)

Hamilton's equations govern the dynamics:

$$\dot{\gamma}_i = \frac{\partial \mathcal{H}}{\partial \mathbf{p}_i}, \qquad \dot{\mathbf{p}}_i = -\frac{\partial \mathcal{H}}{\partial \gamma_i}$$

where $\mathbf{p}_i = m_i \dot{\gamma}_i$ is the momentum of room $i$.

### 3.5 Potential Energy as Accumulated Correlation

The potential energy stored in the $A$–$B$ interaction is:

$$U_{AB}(t) = \int_{t_0}^{t} \dot{\rho}_{AB}(\tau) \, d\tau = \rho_{AB}(t) - \rho_{AB}(t_0)$$

This is the **accumulated correlation** — the total shift in relationship since observation began. A JEPA query like "how has Room A's relationship with Room B changed since last week?" is answered by evaluating $\Delta U_{AB} = U_{AB}(t_{\text{now}}) - U_{AB}(t_{\text{week ago}})$.

---

## 4. The JEPA Moment: Time-Varying Kernel Functions

### 4.1 Cross-Room Similarity Kernel

The JEPA's understanding of cross-room dynamics is encoded in a **time-varying kernel**:

$$K_{AB}(t_1, t_2) = \exp\left(-\frac{\|\gamma_A(t_1) - \gamma_B(t_2)\|^2}{2\sigma^2}\right)$$

This is a Gaussian (RBF) kernel evaluated between Room A at time $t_1$ and Room B at time $t_2$. The kernel satisfies:

- $K_{AB}(t, t) \in [0, 1]$: simultaneous similarity
- $K_{AB}(t_1, t_2) = K_{BA}(t_2, t_1)$: cross-time symmetry
- For fixed $t_1$, $K_{AB}(t_1, \cdot)$ is a function on $[t_0, t_N]$ describing how Room B's trajectory relates to Room A's state at time $t_1$

### 4.2 Temporal Structure of the Kernel

The auto-correlation kernel for a single room,

$$K_A(t_1, t_2) = \exp\left(-\frac{\|\gamma_A(t_1) - \gamma_A(t_2)\|^2}{2\sigma^2}\right)$$

has a natural interpretation: it measures **self-similarity over time**. If the room revisits a previous state, $K_A$ will show a peak. Periodic behavior (daily cycles) produces a checkerboard pattern.

### 4.3 Mercer Decomposition

By Mercer's theorem, the kernel admits a spectral decomposition:

$$K_{AB}(t_1, t_2) = \sum_{\ell=1}^{\infty} \lambda_\ell \, \phi_\ell(t_1) \, \phi_\ell(t_2)$$

where $\{\lambda_\ell, \phi_\ell\}$ are eigenvalue–eigenfunction pairs. The leading eigenfunctions capture the dominant modes of cross-room correlation. A JEPA query reduces to evaluating a truncated expansion:

$$K_{AB}(t_1, t_2) \approx \sum_{\ell=1}^{L} \lambda_\ell \, \phi_\ell(t_1) \, \phi_\ell(t_2)$$

### 4.4 Functional Queries

The JEPA can compute:

- **Causal influence**: $\frac{\partial K_{AB}}{\partial t_2}(t_1, t_2^+) - K_{AB}(t_1, t_2^-)$ — does Room A's past predict Room B's future?
- **Phase lag**: $\arg\max_{\delta} K_{AB}(t, t + \delta)$ — the time offset at which correlation is maximized.
- **Divergence time**: $\inf\{t : K_{AB}(t, t) < \theta\}$ — the moment rooms diverged below threshold $\theta$.

---

## 5. Multi-Scale Analysis

### 5.1 Wavelet Decomposition of the Trajectory

We decompose the embedding trajectory $\gamma(t)$ using a continuous wavelet transform:

$$W_\gamma(a, b) = \frac{1}{\sqrt{a}} \int_{-\infty}^{\infty} \gamma(t) \, \psi^*\!\left(\frac{t - b}{a}\right) dt$$

where $\psi$ is the mother wavelet (e.g., Morlet wavelet $\psi(t) = \pi^{-1/4} e^{i\omega_0 t} e^{-t^2/2}$), $a$ is the scale parameter, and $b$ is the translation parameter.

### 5.2 Scale Interpretation

| Scale | Period | Interpretation |
|-------|--------|----------------|
| $a \in [2^{13}, 2^{16}]$ (seconds) | Weeks | **Stable character** — the room's enduring identity |
| $a \in [2^{10}, 2^{13}]$ | Days | **Operational patterns** — daily routines and cycles |
| $a \in [2^5, 2^{10}]$ | Minutes–Hours | **Contextual shifts** — activities, occupancy changes |
| $a \in [1, 2^5]$ | Seconds | **Immediate reactions** — transient sensor events |

### 5.3 Multi-Scale Reconstruction

Using the inverse wavelet transform:

$$\gamma(t) = \frac{1}{C_\psi} \int_0^{\infty} \int_{-\infty}^{\infty} W_\gamma(a, b) \frac{1}{\sqrt{a}} \psi\!\left(\frac{t-b}{a}\right) \frac{db \, da}{a^2}$$

We can reconstruct at any scale band:

$$\gamma_{\text{character}}(t) = \frac{1}{C_\psi} \int_{a_1}^{a_2} \int_{-\infty}^{\infty} W_\gamma(a, b) \frac{1}{\sqrt{a}} \psi\!\left(\frac{t-b}{a}\right) \frac{db \, da}{a^2}$$

where $[a_1, a_2]$ corresponds to the "weeks" band.

### 5.4 JEPA Scale Selection

The JEPA selects the relevant scale based on query context:

- "What's the room doing right now?" → high-frequency band $a \in [1, 2^5]$
- "Is today unusual?" → mid-frequency band $a \in [2^{10}, 2^{13}]$
- "Has this room changed since we installed it?" → low-frequency band $a \in [2^{13}, 2^{16}]$

Formally, the JEPA maintains a **scale-conditioned representation**:

$$\hat{\gamma}_A(t \mid s) = \frac{1}{C_\psi} \int_{\alpha_1(s)}^{\alpha_2(s)} \int_{-\infty}^{\infty} W_{\gamma_A}(a, b) \frac{1}{\sqrt{a}} \psi\!\left(\frac{t-b}{a}\right) \frac{db \, da}{a^2}$$

where $s$ indexes the scale of interest and $[\alpha_1(s), \alpha_2(s)]$ is the corresponding band.

---

## 6. Continuous Update Equations

### 6.1 The Tick Update Problem

When a new sensor reading $\mathbf{x}_{new}$ arrives at time $t_{N+1}$, producing embedding $\mathbf{v}_{N+1} = f_\theta(\mathbf{x}_{new})$, the room's understanding must update. This is **not** merely appending $\mathbf{v}_{N+1}$ — it is refining the entire spline.

### 6.2 Kalman Filter on the Embedding Manifold

We model the room's state as a random variable on $\mathcal{M}$ with mean $\hat{\gamma}(t)$ and covariance $P(t)$. The Kalman update proceeds as:

**Prediction step:**

$$\hat{\gamma}^{-}(t_{N+1}) = \Phi \, \hat{\gamma}(t_N)$$
$$P^{-}(t_{N+1}) = \Phi \, P(t_N) \, \Phi^\top + Q$$

where $\Phi$ is the state transition (from the spline's dynamical model) and $Q$ is process noise.

**Update step:**

$$\mathbf{K} = P^{-}(t_{N+1}) \, H^\top \left(H \, P^{-}(t_{N+1}) \, H^\top + R\right)^{-1}$$
$$\hat{\gamma}(t_{N+1}) = \hat{\gamma}^{-}(t_{N+1}) + \mathbf{K}\left(\mathbf{v}_{N+1} - H \, \hat{\gamma}^{-}(t_{N+1})\right)$$
$$P(t_{N+1}) = (I - \mathbf{K} H) P^{-}(t_{N+1})$$

where $H$ is the observation matrix and $R$ is observation noise covariance.

### 6.3 Extension to Manifolds: The Extended Kalman Filter

On the manifold $\mathcal{M}$, we linearize around the current estimate using the exponential map:

$$\hat{\gamma}^{-}(t_{N+1}) = \exp_{\hat{\gamma}(t_N)}\left(\Phi \cdot \log_{\hat{\gamma}(t_N)}\!\left(\hat{\gamma}(t_N)\right)\right)$$

The correction occurs in the tangent space:

$$\delta = \mathbf{K} \cdot \text{proj}_{T_{\hat{\gamma}^{-}}\mathcal{M}}\!\left(\mathbf{v}_{N+1} - \hat{\gamma}^{-}\right)$$
$$\hat{\gamma}(t_{N+1}) = \exp_{\hat{\gamma}^{-}(t_{N+1})}(\delta)$$

This is the **manifold Kalman filter** — the spline is refined on the intrinsic geometry of the understanding surface.

### 6.4 Bayesian Update of Belief State

More generally, the room maintains a belief distribution $p(\gamma \mid \mathbf{v}_{0:N})$ over trajectories. Each tick performs a Bayesian update:

$$p(\gamma \mid \mathbf{v}_{0:N+1}) \propto p(\mathbf{v}_{N+1} \mid \gamma(t_{N+1})) \cdot p(\gamma \mid \mathbf{v}_{0:N})$$

For computational tractability, we approximate the belief with a Gaussian process (GP) prior on $\gamma$:

$$\gamma \sim \mathcal{GP}\!\left(\mathbf{m}(t), \, k_{\text{prior}}(t, t')\right)$$

where $k_{\text{prior}}(t, t') = \sigma_f^2 \exp\!\left(-\frac{(t - t')^2}{2\ell^2}\right)$ is a squared-exponential kernel with length-scale $\ell$. The posterior mean $\bar{\gamma}(t)$ is the best spline estimate, and the posterior variance quantifies uncertainty.

### 6.5 Information Geometry: Fisher Information Metric

The Fisher information metric quantifies how much a single tick changes understanding. Given the parametric family $p(\mathbf{v} \mid \theta)$ where $\theta$ parameterizes the room's current understanding:

$$g_{ij}(\theta) = \mathbb{E}\!\left[\frac{\partial \log p(\mathbf{v} \mid \theta)}{\partial \theta_i} \cdot \frac{\partial \log p(\mathbf{v} \mid \theta)}{\partial \theta_j}\right]$$

The Fisher information of tick $N+1$ is:

$$\mathcal{I}_{N+1} = \text{tr}\left(g(\theta_{N+1})\right)$$

A large $\mathcal{I}_{N+1}$ means the tick was **highly informative** — it substantially changed the room's understanding. A small $\mathcal{I}_{N+1}$ means the tick was predictable and added little.

### 6.6 The Complete Tick Update Algorithm

Combining all elements, the tick update proceeds as follows:

```
Algorithm: Tick Update
Input: New sensor reading x_{N+1} at time t_{N+1}
Output: Updated spline γ(t), updated manifold M, updated belief

1. Compute embedding: v_{N+1} = f_θ(x_{N+1})
2. Compute Fisher information: I_{N+1} = tr(g(θ))
3. If I_{N+1} > threshold:
     Flag tick as "significant" for JEPA attention
4. Manifold Kalman update:
     - Predict γ⁻(t_{N+1}) from spline dynamics
     - Compute innovation: ν = v_{N+1} - γ⁻(t_{N+1})
     - Update: γ(t_{N+1}) = γ⁻(t_{N+1}) + K·ν
5. Refine spline:
     - Insert new knot at t_{N+1}
     - Solve local least-squares for nearby control points
     - Propagate refinement to ensure C² continuity
6. Update manifold embedding:
     - Find k-nearest neighbors to v_{N+1}
     - Compute membership strengths
     - Optimize y_{N+1} via cross-entropy minimization
     - Refine nearby manifold points (optional)
7. Update Hamiltonian:
     - Recompute forces F_{AB} for all coupled rooms
     - Integrate equations of motion (symplectic integrator)
8. Compute wavelet coefficients:
     - Update W_γ(a, b) for all scales
9. Return updated state
```

---

## 7. Convergence and Stability Analysis

### 7.1 Spline Refinement Stability

As new knots are inserted, the spline $\gamma_N(t)$ converges to the true underlying trajectory $\gamma^*(t)$ under mild conditions. Specifically, if the maximum knot spacing $h_N = \max_k |t_{k+1} - t_k| \to 0$, then:

$$\|\gamma_N - \gamma^*\|_{L^\infty} = O(h_N^{p+1})$$

for a spline of degree $p$. For cubic splines, this gives fourth-order convergence.

### 7.2 Manifold Consistency

The online UMAP embedding is consistent with the batch embedding under the assumption that new points lie within the convex hull (or a neighborhood thereof) of existing points. For out-of-distribution observations, the manifold may need to grow new branches — analogous to topological changes in the simplicial complex.

### 7.3 Hamiltonian Energy Conservation

The symplectic integrator for the room-room dynamics conserves a shadow Hamiltonian $\tilde{\mathcal{H}}$ to order $O(\Delta t^r)$ where $r$ is the integrator order. For the Störmer–Verlet scheme ($r = 2$):

$$\tilde{\mathcal{H}} = \mathcal{H} + O(\Delta t^2)$$

ensuring long-term stability of the inter-room dynamics.

---

## 8. Summary of Key Equations

| Concept | Equation |
|---------|----------|
| Embedding spline | $\gamma(t) = \sum_i \mathbf{c}_i B_{i,p}(t)$ |
| Curvature | $\kappa(t) = \frac{\sqrt{\|\gamma'\|^2\|\gamma''\|^2 - \langle\gamma',\gamma''\rangle^2}}{\|\gamma'\|^3}$ |
| Geodesic distance | $d_\mathcal{M}(\mathbf{v}_a, \mathbf{v}_b) = \min_{\text{paths}} \sum_{(i,j)} -\log(\mu_{ij})$ |
| Correlation force | $\mathbf{F}_{AB} = \dot{\rho}_{AB}(t) \cdot \hat{n}_{AB}(t)$ |
| Hamiltonian | $\mathcal{H} = \sum_i \frac{1}{2}m_i\|\dot{\gamma}_i\|^2 + \sum_{i<j}V_{ij}(\|\gamma_i-\gamma_j\|)$ |
| Cross-room kernel | $K_{AB}(t_1, t_2) = \exp\!\left(-\frac{\|\gamma_A(t_1)-\gamma_B(t_2)\|^2}{2\sigma^2}\right)$ |
| Wavelet transform | $W_\gamma(a,b) = \frac{1}{\sqrt{a}}\int \gamma(t)\psi^*\!\left(\frac{t-b}{a}\right)dt$ |
| Manifold Kalman update | $\hat{\gamma}(t_{N+1}) = \exp_{\hat{\gamma}^-}(\mathbf{K}\cdot\text{proj}_{T\mathcal{M}}(\mathbf{v}_{N+1}-\hat{\gamma}^-))$ |
| Fisher information | $g_{ij}(\theta) = \mathbb{E}\!\left[\partial_i \log p \cdot \partial_j \log p\right]$ |

---

## References

1. de Boor, C. (1978). *A Practical Guide to Splines*. Springer-Verlag.
2. McInnes, L., Healy, J., & Melville, J. (2018). UMAP: Uniform Manifold Approximation and Projection for dimension reduction. *arXiv:1802.03426*.
3. Kalman, R. E. (1960). A new approach to linear filtering and prediction problems. *Journal of Basic Engineering*, 82(1), 35–45.
4. Mallat, S. (1999). *A Wavelet Tour of Signal Processing*. Academic Press.
5. Amari, S.-I. & Nagaoka, H. (2000). *Methods of Information Geometry*. AMS.
6. LeCun, Y. (2022). A Path Towards Autonomous Machine Intelligence. *Meta AI Technical Report*.
7. Absil, P.-A., Mahony, R., & Sepulchre, R. (2008). *Optimization Algorithms on Matrix Manifolds*. Princeton University Press.
8. Hairer, E., Lubich, C., & Wanner, G. (2006). *Geometric Numerical Integration*. Springer.

---

*This report establishes the mathematical foundation for PLATO's fluid vector space architecture. Each room's understanding evolves as a spline through manifold-valued space, with inter-room dynamics governed by Hamiltonian mechanics and multi-scale analysis enabling queries at any temporal resolution.*
