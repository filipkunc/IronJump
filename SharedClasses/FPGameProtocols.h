//
//  FPGameProtocols.h
//  IronJump
//
//  Created by Filip Kunc on 5/2/10.
//  For license see LICENSE.TXT
//

#import "FPGraphics.h"
#import "FPTexture.h"
#import "FPGameAtlas.h"
#import "FPFont.h"
#import "FPMath.h"
#import "FPXMLWriter.h"

@protocol FPGameObject;

@protocol FPGameProtocol <NSObject>

@property (readwrite, assign) CGPoint inputAcceleration;
@property (readonly) float width, height;
@property (readonly) id<FPGameObject> player;
@property (readonly) NSMutableArray *gameObjects;

- (void)moveWorldWithX:(float)x y:(float)y;

@end

@protocol FPGameObject <NSObject>

@property (readonly) float x, y;
@property (readonly) CGRect rect;
@property (readwrite, assign) BOOL isVisible;
@property (readonly) BOOL isTransparent;
@property (readonly) BOOL isPlatform, isMovable;

- (void)moveWithX:(float)offsetX y:(float)offsetY;
- (void)draw;
- (id<FPGameObject>)duplicateWithOffsetX:(float)offsetX offsetY:(float)offsetY;

@optional

@property (readwrite, assign) int widthSegments, heightSegments;
@property (readonly) id<FPGameObject> nextPart;
@property (readwrite, assign) float moveY;
@property (readwrite, assign) int textureIndex;

#if TARGET_OS_MAC
+ (FPTexture *)loadTextureIfNeeded;
#endif
- (void)updateWithGame:(id<FPGameProtocol>)game;
- (BOOL)collisionLeftRight:(id<FPGameProtocol>)game;

- (void)parseXMLElement:(NSString *)elementName value:(NSString *)value;
- (void)writeToXML:(FPXMLWriter *)writer;

@end
