(function() {var implementors = {};
implementors["nalgebra"] = [{"text":"impl&lt;N, D:&nbsp;<a class=\"trait\" href=\"nalgebra/base/dimension/trait.DimName.html\" title=\"trait nalgebra::base::dimension::DimName\">DimName</a>&gt; <a class=\"trait\" href=\"num_traits/identities/trait.One.html\" title=\"trait num_traits::identities::One\">One</a> for <a class=\"type\" href=\"nalgebra/base/type.MatrixN.html\" title=\"type nalgebra::base::MatrixN\">MatrixN</a>&lt;N, D&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;N: <a class=\"trait\" href=\"nalgebra/base/trait.Scalar.html\" title=\"trait nalgebra::base::Scalar\">Scalar</a> + <a class=\"trait\" href=\"num_traits/identities/trait.Zero.html\" title=\"trait num_traits::identities::Zero\">Zero</a> + <a class=\"trait\" href=\"num_traits/identities/trait.One.html\" title=\"trait num_traits::identities::One\">One</a> + <a class=\"trait\" href=\"nalgebra/trait.ClosedMul.html\" title=\"trait nalgebra::ClosedMul\">ClosedMul</a> + <a class=\"trait\" href=\"nalgebra/trait.ClosedAdd.html\" title=\"trait nalgebra::ClosedAdd\">ClosedAdd</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"nalgebra/base/default_allocator/struct.DefaultAllocator.html\" title=\"struct nalgebra::base::default_allocator::DefaultAllocator\">DefaultAllocator</a>: <a class=\"trait\" href=\"nalgebra/base/allocator/trait.Allocator.html\" title=\"trait nalgebra::base::allocator::Allocator\">Allocator</a>&lt;N, D, D&gt;,&nbsp;</span>","synthetic":false,"types":["nalgebra::base::alias::MatrixN"]},{"text":"impl&lt;N, D:&nbsp;<a class=\"trait\" href=\"nalgebra/base/dimension/trait.DimName.html\" title=\"trait nalgebra::base::dimension::DimName\">DimName</a>&gt; <a class=\"trait\" href=\"num_traits/identities/trait.One.html\" title=\"trait num_traits::identities::One\">One</a> for <a class=\"struct\" href=\"nalgebra/geometry/struct.Rotation.html\" title=\"struct nalgebra::geometry::Rotation\">Rotation</a>&lt;N, D&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;N: <a class=\"trait\" href=\"nalgebra/base/trait.Scalar.html\" title=\"trait nalgebra::base::Scalar\">Scalar</a> + <a class=\"trait\" href=\"num_traits/identities/trait.Zero.html\" title=\"trait num_traits::identities::Zero\">Zero</a> + <a class=\"trait\" href=\"num_traits/identities/trait.One.html\" title=\"trait num_traits::identities::One\">One</a> + <a class=\"trait\" href=\"nalgebra/trait.ClosedAdd.html\" title=\"trait nalgebra::ClosedAdd\">ClosedAdd</a> + <a class=\"trait\" href=\"nalgebra/trait.ClosedMul.html\" title=\"trait nalgebra::ClosedMul\">ClosedMul</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"nalgebra/base/default_allocator/struct.DefaultAllocator.html\" title=\"struct nalgebra::base::default_allocator::DefaultAllocator\">DefaultAllocator</a>: <a class=\"trait\" href=\"nalgebra/base/allocator/trait.Allocator.html\" title=\"trait nalgebra::base::allocator::Allocator\">Allocator</a>&lt;N, D, D&gt;,&nbsp;</span>","synthetic":false,"types":["nalgebra::geometry::rotation::Rotation"]},{"text":"impl&lt;N:&nbsp;<a class=\"trait\" href=\"nalgebra/trait.SimdRealField.html\" title=\"trait nalgebra::SimdRealField\">SimdRealField</a>&gt; <a class=\"trait\" href=\"num_traits/identities/trait.One.html\" title=\"trait num_traits::identities::One\">One</a> for <a class=\"struct\" href=\"nalgebra/geometry/struct.Quaternion.html\" title=\"struct nalgebra::geometry::Quaternion\">Quaternion</a>&lt;N&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;N::<a class=\"type\" href=\"simba/simd/simd_value/trait.SimdValue.html#associatedtype.Element\" title=\"type simba::simd::simd_value::SimdValue::Element\">Element</a>: <a class=\"trait\" href=\"nalgebra/trait.SimdRealField.html\" title=\"trait nalgebra::SimdRealField\">SimdRealField</a>,&nbsp;</span>","synthetic":false,"types":["nalgebra::geometry::quaternion::Quaternion"]},{"text":"impl&lt;N:&nbsp;<a class=\"trait\" href=\"nalgebra/trait.SimdRealField.html\" title=\"trait nalgebra::SimdRealField\">SimdRealField</a>&gt; <a class=\"trait\" href=\"num_traits/identities/trait.One.html\" title=\"trait num_traits::identities::One\">One</a> for <a class=\"type\" href=\"nalgebra/geometry/type.UnitQuaternion.html\" title=\"type nalgebra::geometry::UnitQuaternion\">UnitQuaternion</a>&lt;N&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;N::<a class=\"type\" href=\"simba/simd/simd_value/trait.SimdValue.html#associatedtype.Element\" title=\"type simba::simd::simd_value::SimdValue::Element\">Element</a>: <a class=\"trait\" href=\"nalgebra/trait.SimdRealField.html\" title=\"trait nalgebra::SimdRealField\">SimdRealField</a>,&nbsp;</span>","synthetic":false,"types":["nalgebra::geometry::quaternion::UnitQuaternion"]},{"text":"impl&lt;N:&nbsp;<a class=\"trait\" href=\"nalgebra/trait.SimdRealField.html\" title=\"trait nalgebra::SimdRealField\">SimdRealField</a>&gt; <a class=\"trait\" href=\"num_traits/identities/trait.One.html\" title=\"trait num_traits::identities::One\">One</a> for <a class=\"type\" href=\"nalgebra/geometry/type.UnitComplex.html\" title=\"type nalgebra::geometry::UnitComplex\">UnitComplex</a>&lt;N&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;N::<a class=\"type\" href=\"simba/simd/simd_value/trait.SimdValue.html#associatedtype.Element\" title=\"type simba::simd::simd_value::SimdValue::Element\">Element</a>: <a class=\"trait\" href=\"nalgebra/trait.SimdRealField.html\" title=\"trait nalgebra::SimdRealField\">SimdRealField</a>,&nbsp;</span>","synthetic":false,"types":["nalgebra::geometry::unit_complex::UnitComplex"]},{"text":"impl&lt;N:&nbsp;<a class=\"trait\" href=\"nalgebra/base/trait.Scalar.html\" title=\"trait nalgebra::base::Scalar\">Scalar</a> + <a class=\"trait\" href=\"num_traits/identities/trait.Zero.html\" title=\"trait num_traits::identities::Zero\">Zero</a> + <a class=\"trait\" href=\"nalgebra/trait.ClosedAdd.html\" title=\"trait nalgebra::ClosedAdd\">ClosedAdd</a>, D:&nbsp;<a class=\"trait\" href=\"nalgebra/base/dimension/trait.DimName.html\" title=\"trait nalgebra::base::dimension::DimName\">DimName</a>&gt; <a class=\"trait\" href=\"num_traits/identities/trait.One.html\" title=\"trait num_traits::identities::One\">One</a> for <a class=\"struct\" href=\"nalgebra/geometry/struct.Translation.html\" title=\"struct nalgebra::geometry::Translation\">Translation</a>&lt;N, D&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"nalgebra/base/default_allocator/struct.DefaultAllocator.html\" title=\"struct nalgebra::base::default_allocator::DefaultAllocator\">DefaultAllocator</a>: <a class=\"trait\" href=\"nalgebra/base/allocator/trait.Allocator.html\" title=\"trait nalgebra::base::allocator::Allocator\">Allocator</a>&lt;N, D&gt;,&nbsp;</span>","synthetic":false,"types":["nalgebra::geometry::translation::Translation"]},{"text":"impl&lt;N:&nbsp;<a class=\"trait\" href=\"nalgebra/trait.SimdRealField.html\" title=\"trait nalgebra::SimdRealField\">SimdRealField</a>, D:&nbsp;<a class=\"trait\" href=\"nalgebra/base/dimension/trait.DimName.html\" title=\"trait nalgebra::base::dimension::DimName\">DimName</a>, R:&nbsp;<a class=\"trait\" href=\"nalgebra/geometry/trait.AbstractRotation.html\" title=\"trait nalgebra::geometry::AbstractRotation\">AbstractRotation</a>&lt;N, D&gt;&gt; <a class=\"trait\" href=\"num_traits/identities/trait.One.html\" title=\"trait num_traits::identities::One\">One</a> for <a class=\"struct\" href=\"nalgebra/geometry/struct.Isometry.html\" title=\"struct nalgebra::geometry::Isometry\">Isometry</a>&lt;N, D, R&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;N::<a class=\"type\" href=\"simba/simd/simd_value/trait.SimdValue.html#associatedtype.Element\" title=\"type simba::simd::simd_value::SimdValue::Element\">Element</a>: <a class=\"trait\" href=\"nalgebra/trait.SimdRealField.html\" title=\"trait nalgebra::SimdRealField\">SimdRealField</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"nalgebra/base/default_allocator/struct.DefaultAllocator.html\" title=\"struct nalgebra::base::default_allocator::DefaultAllocator\">DefaultAllocator</a>: <a class=\"trait\" href=\"nalgebra/base/allocator/trait.Allocator.html\" title=\"trait nalgebra::base::allocator::Allocator\">Allocator</a>&lt;N, D&gt;,&nbsp;</span>","synthetic":false,"types":["nalgebra::geometry::isometry::Isometry"]},{"text":"impl&lt;N:&nbsp;<a class=\"trait\" href=\"nalgebra/trait.SimdRealField.html\" title=\"trait nalgebra::SimdRealField\">SimdRealField</a>, D:&nbsp;<a class=\"trait\" href=\"nalgebra/base/dimension/trait.DimName.html\" title=\"trait nalgebra::base::dimension::DimName\">DimName</a>, R&gt; <a class=\"trait\" href=\"num_traits/identities/trait.One.html\" title=\"trait num_traits::identities::One\">One</a> for <a class=\"struct\" href=\"nalgebra/geometry/struct.Similarity.html\" title=\"struct nalgebra::geometry::Similarity\">Similarity</a>&lt;N, D, R&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;N::<a class=\"type\" href=\"simba/simd/simd_value/trait.SimdValue.html#associatedtype.Element\" title=\"type simba::simd::simd_value::SimdValue::Element\">Element</a>: <a class=\"trait\" href=\"nalgebra/trait.SimdRealField.html\" title=\"trait nalgebra::SimdRealField\">SimdRealField</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;R: <a class=\"trait\" href=\"nalgebra/geometry/trait.AbstractRotation.html\" title=\"trait nalgebra::geometry::AbstractRotation\">AbstractRotation</a>&lt;N, D&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"nalgebra/base/default_allocator/struct.DefaultAllocator.html\" title=\"struct nalgebra::base::default_allocator::DefaultAllocator\">DefaultAllocator</a>: <a class=\"trait\" href=\"nalgebra/base/allocator/trait.Allocator.html\" title=\"trait nalgebra::base::allocator::Allocator\">Allocator</a>&lt;N, D&gt;,&nbsp;</span>","synthetic":false,"types":["nalgebra::geometry::similarity::Similarity"]},{"text":"impl&lt;N:&nbsp;<a class=\"trait\" href=\"nalgebra/trait.RealField.html\" title=\"trait nalgebra::RealField\">RealField</a>, D:&nbsp;<a class=\"trait\" href=\"nalgebra/base/dimension/trait.DimNameAdd.html\" title=\"trait nalgebra::base::dimension::DimNameAdd\">DimNameAdd</a>&lt;<a class=\"struct\" href=\"nalgebra/base/dimension/struct.U1.html\" title=\"struct nalgebra::base::dimension::U1\">U1</a>&gt;, C:&nbsp;<a class=\"trait\" href=\"nalgebra/geometry/trait.TCategory.html\" title=\"trait nalgebra::geometry::TCategory\">TCategory</a>&gt; <a class=\"trait\" href=\"num_traits/identities/trait.One.html\" title=\"trait num_traits::identities::One\">One</a> for <a class=\"struct\" href=\"nalgebra/geometry/struct.Transform.html\" title=\"struct nalgebra::geometry::Transform\">Transform</a>&lt;N, D, C&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;<a class=\"struct\" href=\"nalgebra/base/default_allocator/struct.DefaultAllocator.html\" title=\"struct nalgebra::base::default_allocator::DefaultAllocator\">DefaultAllocator</a>: <a class=\"trait\" href=\"nalgebra/base/allocator/trait.Allocator.html\" title=\"trait nalgebra::base::allocator::Allocator\">Allocator</a>&lt;N, <a class=\"type\" href=\"nalgebra/base/dimension/type.DimNameSum.html\" title=\"type nalgebra::base::dimension::DimNameSum\">DimNameSum</a>&lt;D, <a class=\"struct\" href=\"nalgebra/base/dimension/struct.U1.html\" title=\"struct nalgebra::base::dimension::U1\">U1</a>&gt;, <a class=\"type\" href=\"nalgebra/base/dimension/type.DimNameSum.html\" title=\"type nalgebra::base::dimension::DimNameSum\">DimNameSum</a>&lt;D, <a class=\"struct\" href=\"nalgebra/base/dimension/struct.U1.html\" title=\"struct nalgebra::base::dimension::U1\">U1</a>&gt;&gt;,&nbsp;</span>","synthetic":false,"types":["nalgebra::geometry::transform::Transform"]}];
implementors["num_bigint"] = [{"text":"impl <a class=\"trait\" href=\"num_traits/identities/trait.One.html\" title=\"trait num_traits::identities::One\">One</a> for <a class=\"struct\" href=\"num_bigint/struct.BigInt.html\" title=\"struct num_bigint::BigInt\">BigInt</a>","synthetic":false,"types":["num_bigint::bigint::BigInt"]},{"text":"impl <a class=\"trait\" href=\"num_traits/identities/trait.One.html\" title=\"trait num_traits::identities::One\">One</a> for <a class=\"struct\" href=\"num_bigint/struct.BigUint.html\" title=\"struct num_bigint::BigUint\">BigUint</a>","synthetic":false,"types":["num_bigint::biguint::BigUint"]}];
implementors["num_complex"] = [{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"num_traits/trait.Num.html\" title=\"trait num_traits::Num\">Num</a>&gt; <a class=\"trait\" href=\"num_traits/identities/trait.One.html\" title=\"trait num_traits::identities::One\">One</a> for <a class=\"struct\" href=\"num_complex/struct.Complex.html\" title=\"struct num_complex::Complex\">Complex</a>&lt;T&gt;","synthetic":false,"types":["num_complex::Complex"]}];
implementors["num_rational"] = [{"text":"impl&lt;T:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"num_integer/trait.Integer.html\" title=\"trait num_integer::Integer\">Integer</a>&gt; <a class=\"trait\" href=\"num_traits/identities/trait.One.html\" title=\"trait num_traits::identities::One\">One</a> for <a class=\"struct\" href=\"num_rational/struct.Ratio.html\" title=\"struct num_rational::Ratio\">Ratio</a>&lt;T&gt;","synthetic":false,"types":["num_rational::Ratio"]}];
implementors["num_traits"] = [];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()