//
//  FPGameAtlas.h
//  IronJump
//
//  Created by Filip Kunc on 6/21/10.
//  For license see LICENSE.TXT
//

#import "FPTexture.h"

@interface FPGameAtlas : NSObject 
{
	FPTexture *texture;
	FPAtlasVertex *vertexPtr;
	int verticesUsed;
	
	float texCoordNormalizeX;
	float texCoordNormalizeY;
}

@property (readonly) FPTexture *texture;
@property (readonly) int verticesUsed;

+ (FPGameAtlas *)sharedAtlas;

- (id)initWithFile:(NSString *)fileName;
- (void)removeAllTiles;
- (void)drawAllTiles;

- (void)addTile:(CGRect)aTile atPoint:(CGPoint)aPoint;
- (void)addTile:(CGRect)aTile atPoint:(CGPoint)aPoint widthSegments:(int)aWidthSegments heightSegments:(int)aHeightSegments;
- (void)addTileWithoutCheck:(CGRect)aTile atPoint:(CGPoint)aPoint widthSegments:(int)aWidthSegments heightSegments:(int)aHeightSegments;

- (void)addPlayerAtPoint:(CGPoint)aPoint rotation:(float)aRotation;
- (void)addDiamondAtPoint:(CGPoint)aPoint;
- (void)addExitAtPoint:(CGPoint)aPoint;
- (void)addBackgroundWithIndex:(int)index widthSegments:(int)aWidthSegments heightSegments:(int)aHeightSegments;
- (void)addMovableAtPoint:(CGPoint)aPoint widthSegments:(int)aWidthSegments heightSegments:(int)aHeightSegments;
- (void)addPlatformAtPoint:(CGPoint)aPoint widthSegments:(int)aWidthSegments heightSegments:(int)aHeightSegments;
- (void)addTrampoline:(int)index atPoint:(CGPoint)aPoint widthSegments:(int)aWidthSegments heightSegments:(int)aHeightSegments;
- (void)addElevator:(int)index atPoint:(CGPoint)aPoint widthSegments:(int)aWidthSegments heightSegments:(int)aHeightSegments;
- (void)addMagnetAtPoint:(CGPoint)aPoint widthSegments:(int)aWidthSegments;
- (void)addSpeedPowerUpAtPoint:(CGPoint)aPoint;
- (void)addSpeedEffectAtPoint:(CGPoint)aPoint;

@end
